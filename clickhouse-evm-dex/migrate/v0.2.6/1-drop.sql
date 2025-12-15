-- swaps
DROP TABLE IF EXISTS swaps ON CLUSTER 'tokenapis-a'
SETTINGS max_table_size_to_drop = 0;
DROP TABLE IF EXISTS mv_swaps_sunpump_token_purchased ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_swaps_sunpump_token_sold ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_swaps_uniswap_v1_token_purchase ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_swaps_uniswap_v1_eth_purchase ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_swaps_uniswap_v2_swap ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_swaps_uniswap_v3_swap ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_swaps_uniswap_v4_swap ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_swaps_curvefi_token_exchange ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_swaps_balancer_vault_swap ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_swaps_bancor_conversion ON CLUSTER 'tokenapis-a';

-- state fees --
DROP TABLE IF EXISTS state_pools_fees ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_uniswap_v1_new_exchange_fee ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_uniswap_v2_pair_created_fee ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_uniswap_v3_pool_created_fee ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_uniswap_v4_initialize_fee ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_curvefi_plain_pool_deployed_fee ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_curvefi_meta_pool_deployed_fee ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_balancer_swap_fee_percentage ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_balancer_aggregate_swap_fee_percentage ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_bancor_conversion_fee_update ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_curvefi_commit_new_fee ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_curvefi_new_fee ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_sunpump_purchase_fee_set ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_fees_sunpump_sale_fee_set ON CLUSTER 'tokenapis-a';

-- state initialize --
DROP TABLE IF EXISTS state_pools_initialize ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_initialize_uniswap_v2_pair_created ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_initialize_uniswap_v3_pool_created ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_initialize_uniswap_v4_initialize ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_initialize_uniswap_v1_new_exchange ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_initialize_sunpump_token_create ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_initialize_sunpump_token_create_legacy ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_initialize_curvefi_plain_pool_deployed ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_initialize_curvefi_meta_pool_deployed ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_initialize_balancer_pool_registered ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_initialize_bancor_activation ON CLUSTER 'tokenapis-a';

-- state aggregating by pool/token --
DROP TABLE IF EXISTS state_pools_aggregating_by_pool ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_aggregating_by_pool_swaps ON CLUSTER 'tokenapis-a';

DROP TABLE IF EXISTS state_pools_aggregating_by_token ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_aggregating_by_token_input_contract ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_state_pools_aggregating_by_token_output_contract ON CLUSTER 'tokenapis-a';

DROP TABLE IF EXISTS ohlc_prices ON CLUSTER 'tokenapis-a'
SETTINGS max_table_size_to_drop = 0;
DROP TABLE IF EXISTS mv_ohlc_prices_swaps ON CLUSTER 'tokenapis-a';

-- extras
DROP TABLE IF EXISTS mv_balancer_vault_swap ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_bancor_conversion ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_curvefi_token_exchange ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_ohlc_prices ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_sunpump_token_purchased ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_sunpump_token_sold ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_uniswap_v1_eth_purchase ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_uniswap_v1_token_purchase ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_uniswap_v2_swap ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_uniswap_v3_swap ON CLUSTER 'tokenapis-a';
DROP TABLE IF EXISTS mv_uniswap_v4_swap ON CLUSTER 'tokenapis-a';