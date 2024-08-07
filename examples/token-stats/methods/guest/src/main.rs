// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloy_sol_types::SolValue;
use core::{APRCommitment, CometMainInterface, CONTRACT};
use risc0_steel::{
    ethereum::{EthEvmInput, ETH_MAINNET_CHAIN_SPEC},
    Contract,
};
use risc0_zkvm::guest::env;

const SECONDS_PER_YEAR: u64 = 60 * 60 * 24 * 365;

fn main() {
    // Read the input from the guest environment.
    let input: EthEvmInput = env::read();

    // Converts the input into a `EvmEnv` for execution. The `with_chain_spec` method is used
    // to specify the chain configuration. It checks that the state matches the state root in the
    // header provided in the input.
    let env = input.into_env().with_chain_spec(&ETH_MAINNET_CHAIN_SPEC);

    // Execute the view calls; it returns the result in the type generated by the `sol!` macro.
    let contract = Contract::new(CONTRACT, &env);
    let utilization = contract
        .call_builder(&CometMainInterface::getUtilizationCall {})
        .call()
        ._0;
    let supply_rate = contract
        .call_builder(&CometMainInterface::getSupplyRateCall { utilization })
        .call()
        ._0;

    // The formula for APR in percentage is the following:
    // Seconds Per Year = 60 * 60 * 24 * 365
    // Utilization = getUtilization()
    // Supply Rate = getSupplyRate(Utilization)
    // Supply APR = Supply Rate / (10 ^ 18) * Seconds Per Year * 100
    //
    // And this is calculating: Supply Rate * Seconds Per Year, to avoid float calculations for
    // precision.
    let annual_supply_rate = supply_rate * SECONDS_PER_YEAR;

    // This commits the APR at current utilization rate for this given block.
    let journal = APRCommitment {
        commitment: env.into_commitment(),
        annualSupplyRate: annual_supply_rate,
    };
    env::commit_slice(&journal.abi_encode());
}
