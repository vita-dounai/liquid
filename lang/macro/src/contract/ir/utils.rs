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

use super::{Function, ItemAsset, ItemEvent, ItemStorage, LiquidItem, Marker};
use crate::utils as lang_utils;
use proc_macro2::Span;
use syn::{spanned::Spanned, Result};

pub fn filter_map_liquid_attributes<'a, I>(attrs: I) -> Result<Vec<Marker>>
where
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    use core::convert::TryFrom;

    let mut markers = Vec::new();
    for attr in attrs {
        if lang_utils::is_liquid_attribute(attr) {
            let marker = Marker::try_from(attr.clone());
            if let Ok(marker) = marker {
                markers.push(marker);
            } else {
                return Err(marker.unwrap_err());
            }
        }
    }

    Ok(markers)
}

pub type ContractItems = (
    ItemStorage,
    Vec<ItemEvent>,
    Vec<ItemAsset>,
    Vec<Function>,
    Vec<syn::ImplItemConst>,
);

pub fn split_items(items: Vec<LiquidItem>, span: Span) -> Result<ContractItems> {
    use either::Either;
    use itertools::Itertools;

    let (mut storages, others): (Vec<_>, Vec<_>) =
        items.into_iter().partition_map(|item| match item {
            LiquidItem::Storage(storage) => Either::Left(storage),
            other => Either::Right(other),
        });
    let storage = match storages.len() {
        0 => {
            return Err(format_err_span!(
                span,
                "no `#[liquid(storage)]` struct found in this contract"
            ))
        }
        1 => storages.pop().unwrap(),
        _ => {
            return Err(format_err_span!(
                storages[1].span(),
                "duplicate `#[liquid(storage)]` struct definition found here"
            ))
        }
    };
    let (assets, others): (Vec<_>, Vec<_>) =
        others.into_iter().partition_map(|item| match item {
            LiquidItem::Asset(asset) => Either::Left(asset),
            other => Either::Right(other),
        });

    let (events, impl_blocks): (Vec<_>, Vec<_>) =
        others.into_iter().partition_map(|item| match item {
            LiquidItem::Event(event) => Either::Left(event),
            LiquidItem::Impl(impl_block) => Either::Right(impl_block),
            _ => unreachable!(),
        });

    for item_impl in &impl_blocks {
        if item_impl.ty != storage.ident {
            bail!(
                item_impl.ty,
                "liquid impl blocks need to be implemented for the `#[liquid(storage)]` \
                 struct"
            )
        }
    }

    let (functions, constants): (Vec<_>, Vec<_>) = impl_blocks
        .into_iter()
        .map(|block| (block.functions, block.constants))
        .unzip();

    let functions = functions.into_iter().flatten().collect();
    let constants = constants.into_iter().flatten().collect();
    Ok((storage, events, assets, functions, constants))
}
