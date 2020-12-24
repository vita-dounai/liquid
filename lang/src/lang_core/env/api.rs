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

use crate::lang_core::env::{
    backend::Env,
    calldata::CallData,
    engine::{EnvInstance, OnInstance},
    error::Result,
    CallMode,
};
use cfg_if::cfg_if;
use liquid_primitives::{
    types::{timestamp, Address},
    Topics,
};

pub fn set_storage<V>(key: &[u8], value: &V)
where
    V: scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        Env::set_storage::<V>(instance, key, value);
    })
}

pub fn get_storage<R>(key: &[u8]) -> Result<R>
where
    R: scale::Decode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        Env::get_storage::<R>(instance, key)
    })
}

pub fn remove_storage(key: &[u8]) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        Env::remove_storage(instance, key);
    })
}

pub fn get_call_data(mode: CallMode) -> Result<CallData> {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        Env::get_call_data(instance, mode)
    })
}

pub fn get_caller() -> Address {
    <EnvInstance as OnInstance>::on_instance(|instance| Env::get_caller(instance))
}

pub fn get_tx_origin() -> Address {
    <EnvInstance as OnInstance>::on_instance(|instance| Env::get_tx_origin(instance))
}

pub fn get_address() -> Address {
    <EnvInstance as OnInstance>::on_instance(|instance| Env::get_address(instance))
}

pub fn now() -> timestamp {
    <EnvInstance as OnInstance>::on_instance(|instance| Env::now(instance))
}

cfg_if! {
    if #[cfg(feature = "solidity-compatible")] {
        pub fn emit<Event>(event: Event)
        where
            Event: Topics + liquid_abi_codec::Encode,
        {
            <EnvInstance as OnInstance>::on_instance(|instance| {
                Env::emit(instance, event)
            });
        }

        pub fn call<R>(addr: &Address, data: &[u8]) -> Result<R>
        where
            R: liquid_abi_codec::Decode + liquid_abi_codec::TypeInfo,
        {
            <EnvInstance as OnInstance>::on_instance(|instance| {
                Env::call(instance, addr, data)
            })
        }

        pub fn finish<V>(return_value: &V)
        where
            V: liquid_abi_codec::Encode,
        {
            <EnvInstance as OnInstance>::on_instance(|instance| {
                Env::finish(instance, return_value);
            })
        }

        pub fn revert<V>(return_value: &V)
        where
            V: liquid_abi_codec::Encode,
        {
            <EnvInstance as OnInstance>::on_instance(|instance| {
                Env::revert(instance, return_value);
            })
        }
    } else {
        pub fn emit<Event>(event: Event)
        where
            Event: Topics + scale::Encode,
        {
            <EnvInstance as OnInstance>::on_instance(|instance| {
                Env::emit(instance, event)
            });
        }

        pub fn call<R>(addr: &Address, data: &[u8]) -> Result<R>
        where
            R: scale::Decode,
        {
            <EnvInstance as OnInstance>::on_instance(|instance| {
                Env::call(instance, addr, data)
            })
        }

        pub fn finish<V>(return_value: &V)
        where
            V: scale::Encode,
        {
            <EnvInstance as OnInstance>::on_instance(|instance| {
                Env::finish(instance, return_value);
            })
        }

        pub fn revert<V>(return_value: &V)
        where
            V: scale::Encode,
        {
            <EnvInstance as OnInstance>::on_instance(|instance| {
                Env::revert(instance, return_value);
            })
        }
    }
}