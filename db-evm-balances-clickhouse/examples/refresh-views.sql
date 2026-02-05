-- Check last refresh result
SELECT database, view, status, progress, last_refresh_time, next_refresh_time FROM system.view_refreshes;

-- Manually trigger refresh and watch
SYSTEM REFRESH VIEW erc20_balances_final;