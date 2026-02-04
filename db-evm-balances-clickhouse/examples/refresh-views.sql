-- Check last refresh result
SELECT * FROM system.view_refreshes FORMAT Vertical;

-- Manually trigger refresh and watch
SYSTEM REFRESH VIEW erc20_balances_final;