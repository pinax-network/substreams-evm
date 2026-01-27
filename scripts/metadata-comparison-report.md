# Metadata Comparison Report: Legacy vs New

**Date:** January 27, 2026

## Connection Details

| Database  | Database Name |
|----------|---------------|
| Legacy | mainnet:evm-tokens@v1.17.4 |
| New  | metadata |

---

## 📊 Total Records

| Database | Total Records | Unique Contracts | Latest Timestamp |
|----------|--------------|------------------|------------------|
| **Legacy** | 1,439,651 | 1,439,651 | 2026-01-27 19:18:23 |
| **New** | 1,437,520 | 1,437,520 | 2026-01-27 19:18:23 |
| **Difference** | -2,131 | -2,131 | — |

---

## 🔍 Contract Coverage

| Metric | Count |
|--------|-------|
| **Only in Legacy** | 2,431 |
| **Only in New** | 300 |
| **In Both** | 1,437,220 |

---

## 📈 Data Quality Issues

### 1. Empty/NULL Fields

| Issue | Legacy | New | Winner |
|-------|--------|-----|--------|
| Empty name | 3,794 | 5,407 | **Legacy** ✅ |
| Empty symbol | 3,651 | 6,341 | **Legacy** ✅ |
| Zero decimals | 27,065 | 25,515 | **New** ✅ |

### 2. Invalid Records (block_num=0, epoch timestamp)

| Issue | Legacy | New | Winner |
|-------|--------|-----|--------|
| Zero block_num | 1,440 | 1 | **New** ✅ |
| Epoch timestamp (1970-01-01) | 1,440 | 1 | **New** ✅ |

### 3. String Formatting Issues

| Issue | Legacy | New | Winner |
|-------|--------|-----|--------|
| Name with leading/trailing whitespace | 4 | 12,336 | **Legacy** ✅ |
| Symbol with leading/trailing whitespace | 2 | 8,627 | **Legacy** ✅ |
| Name with NULL bytes (`\0`) | 39 | 386 | **Legacy** ✅ |
| Symbol with NULL bytes (`\0`) | 32 | 406 | **Legacy** ✅ |

---

## 🔄 Metadata Consistency (for contracts in both databases)

| Metric | Count | Percentage |
|--------|-------|------------|
| **Identical rows** (contract, name, symbol, decimals) | 1,410,904 | ~98% |
| **Different metadata** | ~26,000 | ~2% |

---

## 🏆 Quality Assessment Summary

| Category | Winner | Notes |
|----------|--------|-------|
| **More contracts** | Legacy | +2,131 more contracts |
| **Cleaner block/timestamp data** | New | Almost no invalid records |
| **Better string formatting** | Legacy | Far fewer whitespace/NULL issues |
| **Less empty name/symbol** | Legacy | 30% fewer empty fields |

---

## 📋 Verdict

### Legacy is currently higher quality overall:
- ✅ Has **2,131 more contracts**
- ✅ Much **cleaner string data** (no excessive whitespace/NULL bytes)
- ✅ **Fewer empty** name/symbol fields

### New database issues to fix:
1. **12,336 names** and **8,627 symbols** have leading/trailing whitespace → apply `trim()`
2. **386 names** and **406 symbols** contain NULL bytes → strip `\0` characters
3. Missing **~2,131 contracts** that Legacy has (though some are questionable with block_num=0)

---

## 🎯 Top 10,000 Most-Transferred Contracts

To measure **practical quality impact**, we compared metadata coverage for the top 10,000 contracts by transfer count (from the `mainnet:evm-transfers@v0.2.2` database).

### Coverage Results

| Database | Contracts with Metadata | Coverage |
|----------|------------------------|----------|
| **Legacy** | 9,920 | **99.20%** |
| **New** | 9,912 | **99.12%** |
| **Difference** | -8 | -0.08% |

### Batch Breakdown

| Batch | Contracts | Legacy | New |
|-------|-----------|--------|-----|
| 0 | 1-1000 | 990 | 989 |
| 1 | 1001-2000 | 992 | 992 |
| 2 | 2001-3000 | 996 | 992 |
| 3 | 3001-4000 | 991 | 991 |
| 4 | 4001-5000 | 998 | 998 |
| 5 | 5001-6000 | 991 | 992 |
| 6 | 6001-7000 | 992 | 991 |
| 7 | 7001-8000 | 989 | 988 |
| 8 | 8001-9000 | 990 | 989 |
| 9 | 9001-10000 | 991 | 990 |
| **Total** | | **9,920** | **9,912** |

### Key Insight
Both databases have **excellent coverage** (~99%) for the most important tokens. Legacy has a slight edge with 8 more contracts covered among the top 10,000 most transferred tokens.

---

## 🛠️ Recommended Fixes for New Database

```sql
-- Fix whitespace issues
ALTER TABLE metadata ON CLUSTER 'tokenapis-b' UPDATE name = trim(name) WHERE name != trim(name);
ALTER TABLE metadata ON CLUSTER 'tokenapis-b' UPDATE symbol = trim(symbol) WHERE symbol != trim(symbol);

-- Fix NULL byte issues
ALTER TABLE metadata ON CLUSTER 'tokenapis-b' UPDATE name = replaceAll(name, '\0', '') WHERE position(name, '\0') > 0;
ALTER TABLE metadata ON CLUSTER 'tokenapis-b' UPDATE symbol = replaceAll(symbol, '\0', '') WHERE position(symbol, '\0') > 0;
```

---

## 📝 Schema Comparison

### Legacy (`metadata_view`)
```
contract        String
block_num       UInt32
timestamp       DateTime('UTC')
decimals        UInt8
name            Nullable(String)
symbol          Nullable(String)
```

### New (`metadata`)
```
contract        String
block_num       UInt32
timestamp       DateTime('UTC')
network         String
decimals        UInt8
name            String
symbol          String
created_at      DateTime('UTC')
```
