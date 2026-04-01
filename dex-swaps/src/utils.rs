pub(crate) fn is_non_zero(value: &str) -> bool {
    !value.is_empty() && value.bytes().any(|byte| byte != b'0')
}
