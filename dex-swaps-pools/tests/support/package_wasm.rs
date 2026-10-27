//! Bounded test-only host that runs the module graph embedded in a packed
//! `.spkg` (an `sf.substreams.v1.Package`). Each module's inputs are read
//! from the package, so a rewired manifest changes what runs. Unsupported
//! newer module fields (including block filters) fail instead of being ignored.
//! No WASI, network or filesystem; 64 MiB of memory, 4 MiB per input and
//! output, bounded fuel. Maps forbid store writes; `run_store` collects bounded
//! `set_if_not_exists` calls without implementing persistent state.
//!
//! Adapted from the SVM package host to the workspace's Prost/Substreams 0.7 ABI.
#![allow(dead_code)]
use prost::Message;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use substreams::pb::substreams::{self as modules, module::input::Input};
use wasmi::{Caller, Config, Engine, Error, Linker, Module, Store, StoreLimits, StoreLimitsBuilder, Val};

const MAX_BYTES: usize = 4 * 1024 * 1024;
const MAX_KEY_BYTES: usize = 1024;
const MAX_STORE_WRITES: usize = 16_384;
const FUEL: u64 = 4_000_000_000;

/// One requested write. The caller applies first-write semantics across blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreWrite {
    pub ordinal: u64,
    pub key: String,
    pub value: Vec<u8>,
}

struct Host {
    limits: StoreLimits,
    output: Option<Vec<u8>>,
    panicked: bool,
    allow_store_writes: bool,
    writes: Vec<StoreWrite>,
    write_bytes: usize,
}

/// Fields of one protobuf message: (number, length-delimited bytes, varint).
fn fields(mut bytes: &[u8]) -> Vec<(u64, &[u8], u64)> {
    fn varint(bytes: &mut &[u8]) -> u64 {
        let (mut value, mut shift) = (0u64, 0);
        loop {
            let byte = bytes[0];
            *bytes = &bytes[1..];
            value |= u64::from(byte & 0x7f) << shift;
            shift += 7;
            if byte < 0x80 {
                return value;
            }
        }
    }
    let mut out = Vec::new();
    while !bytes.is_empty() {
        let key = varint(&mut bytes);
        match key & 7 {
            0 => out.push((key >> 3, &[][..], varint(&mut bytes))),
            1 => bytes = &bytes[8..],
            2 => {
                let length = varint(&mut bytes) as usize;
                out.push((key >> 3, &bytes[..length], 0));
                bytes = &bytes[length..];
            }
            5 => bytes = &bytes[4..],
            other => panic!("unsupported wire type {other}"),
        }
    }
    out
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub struct Package {
    pub path: PathBuf,
    pub sha256: String,
    /// (name, version) of each `package_meta` entry; the first is the package
    /// itself, the others its imports.
    pub meta: Vec<(String, String)>,
    pub descriptors: Vec<prost_types::FileDescriptorProto>,
    engine: Engine,
    binaries: Vec<Module>,
    binary_sha256: Vec<String>,
    modules: HashMap<String, modules::Module>,
}

impl Package {
    pub fn open(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let spkg = std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let mut config = Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let package = fields(&spkg);
        let (_, encoded, _) = *package.iter().find(|(number, _, _)| *number == 6).expect("Package.modules");
        // SDK 0.7's generated Module predates block filters. Refuse unknown
        // module/input fields so a later manifest feature cannot silently
        // evade the graph evaluation performed by this deliberately small host.
        for (_, raw_module, _) in fields(encoded).into_iter().filter(|field| field.0 == 1) {
            for (number, value, _) in fields(raw_module) {
                assert!((1..=8).contains(&number), "unsupported packed Module field {number}");
                if number == 6 {
                    for (input, _, _) in fields(value) {
                        assert!((1..=3).contains(&input), "unsupported packed Input field {input}");
                    }
                }
            }
        }
        let decoded = modules::Modules::decode(encoded).unwrap();
        let meta = package
            .iter()
            .filter(|(number, _, _)| *number == 8)
            .map(|(_, value, _)| {
                let meta = fields(value);
                let text = |n: u64| {
                    meta.iter()
                        .find(|f| f.0 == n)
                        .map(|f| String::from_utf8(f.1.to_vec()).unwrap())
                        .unwrap_or_default()
                };
                (text(3), text(1))
            })
            .collect();
        Self {
            sha256: sha256(&spkg),
            descriptors: package
                .iter()
                .filter(|field| field.0 == 1)
                .map(|field| prost_types::FileDescriptorProto::decode(field.1).unwrap())
                .collect(),
            meta,
            binaries: decoded
                .binaries
                .iter()
                .map(|binary| Module::new(&engine, &binary.content[..]).unwrap())
                .collect(),
            binary_sha256: decoded.binaries.iter().map(|binary| sha256(&binary.content)).collect(),
            modules: decoded.modules.into_iter().map(|module| (module.name.clone(), module)).collect(),
            engine,
            path,
        }
    }

    pub fn module_names(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.modules.keys().map(String::as_str).collect();
        names.sort();
        names
    }

    pub fn has_module(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    pub fn module(&self, name: &str) -> &modules::Module {
        self.modules.get(name).unwrap_or_else(|| panic!("module {name} not in {}", self.path.display()))
    }

    /// SHA-256 of the binary a module runs.
    pub fn binary_of(&self, name: &str) -> &str {
        &self.binary_sha256[self.module(name).binary_index as usize]
    }

    /// A module's wiring as text: its entrypoint, inputs and block filter.
    /// `strip` removes an import prefix from referenced module names.
    pub fn wiring(&self, name: &str, strip: &str) -> String {
        let module = self.module(name);
        let local = |name: &str| name.strip_prefix(strip).unwrap_or(name).to_string();
        let inputs: Vec<String> = module
            .inputs
            .iter()
            .map(|input| match input.input.as_ref().expect("module input") {
                Input::Source(source) => format!("source {}", source.r#type),
                Input::Map(map) => format!("map {}", local(&map.module_name)),
                Input::Store(store) => {
                    use modules::module::input::store::Mode;
                    let mode = match Mode::try_from(store.mode).expect("known store input mode") {
                        Mode::Unset => "unset",
                        Mode::Get => "get",
                        Mode::Deltas => "deltas",
                    };
                    format!("store {} {mode}", local(&store.module_name))
                }
            })
            .collect();
        format!("{} <- [{}]", module.binary_entrypoint, inputs.join(", "))
    }

    /// Runs one map or index module on explicit inputs; `None` is a
    /// registered package panic (a handler error).
    pub fn run(&self, name: &str, inputs: &[&[u8]]) -> Option<Vec<u8>> {
        self.run_with_store_writes(name, inputs, false).map(|(output, _)| output)
    }

    /// Runs one `set_if_not_exists` store handler on explicit inputs. Calls are
    /// collected in execution order; no writes survive a registered panic.
    pub fn run_store(&self, name: &str, inputs: &[&[u8]]) -> Option<Vec<StoreWrite>> {
        use modules::module::{kind_store::UpdatePolicy, Kind};
        let Some(Kind::KindStore(kind)) = &self.module(name).kind else {
            panic!("{name}: expected a store module");
        };
        assert_eq!(
            kind.update_policy,
            UpdatePolicy::SetIfNotExists as i32,
            "{name}: unsupported store update policy"
        );
        self.run_with_store_writes(name, inputs, true).map(|(_, writes)| writes)
    }

    fn run_with_store_writes(&self, name: &str, inputs: &[&[u8]], allow_store_writes: bool) -> Option<(Vec<u8>, Vec<StoreWrite>)> {
        let definition = self.module(name);
        let module = &self.binaries[definition.binary_index as usize];
        let mut store = Store::new(
            &self.engine,
            Host {
                limits: StoreLimitsBuilder::new().memory_size(64 * 1024 * 1024).instances(1).memories(1).build(),
                output: None,
                panicked: false,
                allow_store_writes,
                writes: Vec::new(),
                write_bytes: 0,
            },
        );
        store.limiter(|host| &mut host.limits);
        store.set_fuel(FUEL).unwrap();
        let mut linker = Linker::<Host>::new(&self.engine);
        linker.func_wrap("logger", "println", |_pointer: i32, _length: i32| {}).unwrap();
        linker.func_wrap("env", "skip_empty_output", || {}).unwrap();
        linker
            .func_wrap(
                "env",
                "register_panic",
                |mut caller: Caller<'_, Host>, _: i32, _: i32, _: i32, _: i32, _: i32, _: i32| {
                    caller.data_mut().panicked = true;
                },
            )
            .unwrap();
        linker
            .func_wrap(
                "env",
                "output",
                |mut caller: Caller<'_, Host>, pointer: i32, length: i32| -> Result<(), Error> {
                    if caller.data().allow_store_writes {
                        return Err(Error::new("store handler emitted output"));
                    }
                    let pointer = usize::try_from(pointer).map_err(|_| Error::new("negative output pointer"))?;
                    let length = usize::try_from(length).map_err(|_| Error::new("negative output length"))?;
                    if length > MAX_BYTES || caller.data().output.is_some() {
                        return Err(Error::new("oversized or duplicate output"));
                    }
                    let memory = caller
                        .get_export("memory")
                        .and_then(|value| value.into_memory())
                        .ok_or_else(|| Error::new("missing memory"))?;
                    let mut bytes = vec![0; length];
                    memory.read(&caller, pointer, &mut bytes).map_err(|_| Error::new("invalid output range"))?;
                    caller.data_mut().output = Some(bytes);
                    Ok(())
                },
            )
            .unwrap();
        // Register this for maps as well: a shared binary can import state
        // functions even when the selected handler never calls them.
        linker
            .func_wrap(
                "state",
                "set_if_not_exists",
                |mut caller: Caller<'_, Host>, ordinal: i64, key_pointer: i32, key_length: i32, value_pointer: i32, value_length: i32| -> Result<(), Error> {
                    if !caller.data().allow_store_writes {
                        return Err(Error::new("store write from map or index handler"));
                    }
                    let key_pointer = usize::try_from(key_pointer).map_err(|_| Error::new("negative store key pointer"))?;
                    let key_length = usize::try_from(key_length).map_err(|_| Error::new("negative store key length"))?;
                    let value_pointer = usize::try_from(value_pointer).map_err(|_| Error::new("negative store value pointer"))?;
                    let value_length = usize::try_from(value_length).map_err(|_| Error::new("negative store value length"))?;
                    let write_bytes = caller
                        .data()
                        .write_bytes
                        .checked_add(key_length)
                        .and_then(|total| total.checked_add(value_length))
                        .ok_or_else(|| Error::new("store write size overflow"))?;
                    if key_length > MAX_KEY_BYTES || value_length > MAX_BYTES || write_bytes > MAX_BYTES || caller.data().writes.len() >= MAX_STORE_WRITES {
                        return Err(Error::new("store write limit exceeded"));
                    }
                    let memory = caller
                        .get_export("memory")
                        .and_then(|value| value.into_memory())
                        .ok_or_else(|| Error::new("missing memory"))?;
                    let mut key = vec![0; key_length];
                    let mut value = vec![0; value_length];
                    memory.read(&caller, key_pointer, &mut key).map_err(|_| Error::new("invalid store key range"))?;
                    memory
                        .read(&caller, value_pointer, &mut value)
                        .map_err(|_| Error::new("invalid store value range"))?;
                    let key = String::from_utf8(key).map_err(|_| Error::new("invalid UTF-8 store key"))?;
                    let host = caller.data_mut();
                    // The SDK casts u64 to i64 for the WASM ABI; retain its bits.
                    host.writes.push(StoreWrite {
                        ordinal: ordinal as u64,
                        key,
                        value,
                    });
                    host.write_bytes = write_bytes;
                    Ok(())
                },
            )
            .unwrap();
        // `+wasm-bindgen-shims` binaries link these imports; the handlers
        // under test must never call them.
        for import in module.imports().filter(|import| import.module().starts_with("__wbindgen_")) {
            linker
                .func_new(import.module(), import.name(), import.ty().func().unwrap().clone(), |_, _, _| {
                    Err(Error::new("unexpected JavaScript host invocation"))
                })
                .unwrap();
        }
        let instance = linker.instantiate(&mut store, module).unwrap().start(&mut store).unwrap();
        let allocate = instance.get_typed_func::<i32, i32>(&store, "alloc").unwrap();
        let memory = instance.get_memory(&store, "memory").unwrap();
        let mut arguments = Vec::with_capacity(inputs.len() * 2);
        for bytes in inputs {
            assert!(bytes.len() <= MAX_BYTES, "{name}: input of {} bytes", bytes.len());
            let length = i32::try_from(bytes.len()).unwrap();
            let pointer = allocate.call(&mut store, length).unwrap();
            memory.write(&mut store, usize::try_from(pointer).unwrap(), bytes).unwrap();
            arguments.extend([Val::I32(pointer), Val::I32(length)]);
        }
        let result = instance
            .get_func(&store, &definition.binary_entrypoint)
            .unwrap_or_else(|| panic!("{name}: no export {}", definition.binary_entrypoint))
            .call(&mut store, &arguments, &mut []);
        if store.data().panicked {
            assert!(result.is_err(), "a registered panic must trap");
            None
        } else {
            result.unwrap_or_else(|error| panic!("{name}: unexpected WASM host/trap/fuel failure: {error}"));
            let host = store.data_mut();
            Some((host.output.take().unwrap_or_default(), std::mem::take(&mut host.writes)))
        }
    }

    /// Evaluate only map/source edges from the actual packed graph. Store inputs
    /// are deliberately supplied explicitly to `run`, never synthesized here.
    pub fn evaluate(&self, name: &str, block: &[u8]) -> Vec<u8> {
        self.evaluate_memo(name, block, &mut HashMap::new(), &mut Vec::new())
    }

    fn evaluate_memo(&self, name: &str, block: &[u8], memo: &mut HashMap<String, Vec<u8>>, stack: &mut Vec<String>) -> Vec<u8> {
        if let Some(output) = memo.get(name) {
            return output.clone();
        }
        assert!(!stack.iter().any(|node| node == name), "module graph cycle at {name}");
        stack.push(name.to_string());
        let inputs: Vec<Vec<u8>> = self
            .module(name)
            .inputs
            .iter()
            .map(|input| match input.input.as_ref().expect("module input") {
                Input::Source(source) => {
                    assert_eq!(source.r#type, "sf.ethereum.type.v2.Block", "{name}: unexpected source");
                    block.to_vec()
                }
                Input::Map(map) => self.evaluate_memo(&map.module_name, block, memo, stack),
                Input::Store(_) => panic!("{name}: store deltas require explicit input"),
            })
            .collect();
        let inputs: Vec<&[u8]> = inputs.iter().map(Vec::as_slice).collect();
        let output = self.run(name, &inputs).unwrap_or_else(|| panic!("{name}: package panic"));
        stack.pop();
        memo.insert(name.to_string(), output.clone());
        output
    }
}
