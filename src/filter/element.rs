//! Keeps track of rules applied on attributes or tags. They can
//! either be blacklisted or whitelisted by the user. This module handles the
//! logic for the combination of these rules.

use std::collections::HashMap;

use crate::types::tag::Attribute;

/// Stores the status of an element, i.e., whether it ought to be kept or
/// removed.
///
/// This contains only the explicit rules given by the user at the definition of
/// [`super::Filter`].
///
/// It contains a `whitelist` and a `blacklist` to keep track of the filtering
/// parameters.
#[derive(Debug)]
pub struct BlackWhiteList {
    /// Default behaviour
    ///
    /// Only is used when checking for emptiness
    default: bool,
    /// Contains the elements and their status
    ///
    /// The hashmap maps a name to a target, and a bool. The boolean is `true`
    /// if the item is whitelisted, and `false` if the item is blacklisted.
    items: HashMap<String, bool>,
    /// Indicates if a whitelisted element was pushed into the [`HashMap`].
    whitelist_empty: bool,
}

impl BlackWhiteList {
    /// Check the status of an element
    pub fn check(&self, name: &String) -> ElementState {
        self.items.get(name).map_or_else(
            || {
                if self.is_empty() && self.default {
                    ElementState::NotSpecified
                } else {
                    ElementState::BlackListed
                }
            },
            |keep| match keep {
                true => ElementState::WhiteListed,
                false => ElementState::BlackListed,
            },
        )
    }

    /// Checks if no elements were specified
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.whitelist_empty
    }

    /// Pushes an element as whitelisted or blacklisted
    pub fn push(&mut self, name: String, keep: bool) -> Result<(), ()> {
        if keep {
            self.whitelist_empty = false;
        }
        let old = self.items.insert(name, keep);
        if old.is_some_and(|inner| inner != keep) {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Sets the default rule
    ///
    /// If no rule is specified for the given tag, default is applied.
    pub const fn set_default(&mut self, default: bool) {
        self.default = default;
    }
}

impl Default for BlackWhiteList {
    fn default() -> Self {
        Self { items: HashMap::new(), whitelist_empty: true, default: true }
    }
}

/// Status of an element
///
/// An element can be whitelisted or blacklisted by the user. This state
/// contains both information.
#[derive(Debug)]
pub enum ElementState {
    /// Element ought to be removed
    BlackListed,
    /// No rules applied for this element
    NotSpecified,
    /// Element ought to be kept
    WhiteListed,
}

impl ElementState {
    /// Computes the output status for multiple checks
    ///
    /// This is used to perform multiple successive tests.
    pub const fn and(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::BlackListed, _) | (_, Self::BlackListed) => Self::BlackListed,
            (Self::NotSpecified, Self::NotSpecified) => Self::NotSpecified,
            // in this arm, at least one is WhiteListed, because the other case is above.
            (Self::WhiteListed | Self::NotSpecified, Self::WhiteListed | Self::NotSpecified) =>
                Self::WhiteListed,
        }
    }

    /// Checks if an element was explicitly authorised, i.e., is whitelisted
    pub const fn is_allowed_or(&self, default: bool) -> bool {
        match self {
            Self::BlackListed => false,
            Self::NotSpecified => default,
            Self::WhiteListed => true,
        }
    }
}

/// Rules for associating names to values
//TODO: could add a default to create a method: exact_attributes
#[derive(Default, Debug)]
pub struct ValueAssociateHash {
    /// Names and attributes explicitly not wanted
    blacklist: Vec<(String, Option<String>)>,
    /// Names and attributes explicitly wanted
    whitelist: Vec<(String, Option<String>)>,
}

impl ValueAssociateHash {
    /// Checks if the attributes form a correct combination of rules
    pub fn check(&self, attrs: &[Attribute]) -> ElementState {
        let attrs_map: HashMap<_, _> = attrs
            .iter()
            .map(|attr| (attr.as_name().to_string(), attr.as_value()))
            .collect();
        for (wanted_name, wanted_value) in &self.whitelist {
            match attrs_map.get(wanted_name) {
                None => return ElementState::BlackListed,
                Some(found_value) if *found_value != wanted_value.as_ref() =>
                    return ElementState::BlackListed,
                Some(_) => (),
            }
        }
        for (wanted_name, wanted_value) in &self.blacklist {
            match attrs_map.get(wanted_name) {
                Some(found_value) if *found_value == wanted_value.as_ref() =>
                    return ElementState::BlackListed,
                Some(_) | None => (),
            }
        }
        if self.is_empty() {
            ElementState::NotSpecified
        } else {
            ElementState::WhiteListed
        }
    }

    /// Checks if the [`ValueAssociateHash`] wasn't given any rules.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.whitelist.is_empty() && self.blacklist.is_empty()
    }

    /// Adds a rule for the attribute `name`
    pub fn push(&mut self, name: String, value: Option<String>, keep: bool) {
        let () = if keep {
            self.whitelist.push((name, value));
        } else {
            self.blacklist.push((name, value));
        };
    }
}
