#![cfg(not(feature = "no_object"))]

use crate::engine::OP_EQUALS;
use crate::plugin::*;
use crate::{def_package, Dynamic, ImmutableString, Map, RhaiResultOf, INT};
#[cfg(feature = "no_std")]
use std::prelude::v1::*;

#[cfg(not(feature = "no_index"))]
use crate::Array;

def_package! {
    /// Package of basic object map utilities.
    crate::BasicMapPackage => |lib| {
        lib.standard = true;

        combine_with_exported_module!(lib, "map", map_functions);
    }
}

#[export_module]
mod map_functions {
    /// Return the number of properties in the object map.
    #[rhai_fn(pure)]
    pub fn len(map: &mut Map) -> INT {
        map.len() as INT
    }
    /// Clear the object map.
    pub fn clear(map: &mut Map) {
        if !map.is_empty() {
            map.clear();
        }
    }
    /// Remove any property of the specified `name` from the object map, returning its value.
    ///
    /// If the property does not exist, `()` is returned.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let m = #{a:1, b:2, c:3};
    ///
    /// let x = m.remove("b");
    ///
    /// print(x);       // prints 2
    ///
    /// print(m);       // prints "#{a:1, c:3}"
    /// ```
    pub fn remove(map: &mut Map, name: ImmutableString) -> Dynamic {
        if !map.is_empty() {
            map.remove(name.as_str()).unwrap_or_else(|| Dynamic::UNIT)
        } else {
            Dynamic::UNIT
        }
    }
    /// Add all property values of another object map into the object map.
    /// Existing property values of the same names are replaced.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let m = #{a:1, b:2, c:3};
    /// let n = #{a: 42, d:0};
    ///
    /// m.mixin(n);
    ///
    /// print(m);       // prints "#{a:42, b:2, c:3, d:0}"
    /// ```
    #[rhai_fn(name = "mixin", name = "+=")]
    pub fn mixin(map: &mut Map, map2: Map) {
        if !map2.is_empty() {
            map.extend(map2.into_iter());
        }
    }
    /// Make a copy of the object map, add all property values of another object map
    /// (existing property values of the same names are replaced), then returning it.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let m = #{a:1, b:2, c:3};
    /// let n = #{a: 42, d:0};
    ///
    /// print(m + n);       // prints "#{a:42, b:2, c:3, d:0}"
    ///
    /// print(m);           // prints "#{a:1, b:2, c:3}"
    /// ```
    #[rhai_fn(name = "+")]
    pub fn merge(map1: Map, map2: Map) -> Map {
        if map2.is_empty() {
            map1
        } else if map1.is_empty() {
            map2
        } else {
            let mut map1 = map1;
            map1.extend(map2.into_iter());
            map1
        }
    }
    /// Add all property values of another object map into the object map.
    /// Only properties that do not originally exist in the object map are added.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let m = #{a:1, b:2, c:3};
    /// let n = #{a: 42, d:0};
    ///
    /// m.fill_with(n);
    ///
    /// print(m);       // prints "#{a:1, b:2, c:3, d:0}"
    /// ```
    pub fn fill_with(map: &mut Map, map2: Map) {
        if !map2.is_empty() {
            if map.is_empty() {
                *map = map2;
            } else {
                map2.into_iter().for_each(|(key, value)| {
                    map.entry(key).or_insert(value);
                });
            }
        }
    }
    /// Return `true` if two object maps are equal (i.e. all property values are equal).
    ///
    /// The operator `==` is used to compare property values and must be defined,
    /// otherwise `false` is assumed.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let m1 = #{a:1, b:2, c:3};
    /// let m2 = #{a:1, b:2, c:3};
    /// let m3 = #{a:1, c:3};
    ///
    /// print(m1 == m2);        // prints true
    ///
    /// print(m1 == m3);        // prints false
    /// ```
    #[rhai_fn(name = "==", return_raw, pure)]
    pub fn equals(ctx: NativeCallContext, map1: &mut Map, map2: Map) -> RhaiResultOf<bool> {
        if map1.len() != map2.len() {
            return Ok(false);
        }

        if !map1.is_empty() {
            let mut map2 = map2;

            for (m1, v1) in map1.iter_mut() {
                if let Some(v2) = map2.get_mut(m1) {
                    let equals = ctx
                        .call_fn_raw(OP_EQUALS, true, false, &mut [v1, v2])?
                        .as_bool()
                        .unwrap_or(false);

                    if !equals {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
    /// Return `true` if two object maps are not equal (i.e. at least one property value is not equal).
    ///
    /// The operator `==` is used to compare property values and must be defined,
    /// otherwise `false` is assumed.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let m1 = #{a:1, b:2, c:3};
    /// let m2 = #{a:1, b:2, c:3};
    /// let m3 = #{a:1, c:3};
    ///
    /// print(m1 != m2);        // prints false
    ///
    /// print(m1 != m3);        // prints true
    /// ```
    #[rhai_fn(name = "!=", return_raw, pure)]
    pub fn not_equals(ctx: NativeCallContext, map1: &mut Map, map2: Map) -> RhaiResultOf<bool> {
        equals(ctx, map1, map2).map(|r| !r)
    }

    /// Return an array with all the property names in the object map.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let m = #{a:1, b:2, c:3};
    ///
    /// print(m.keys());        // prints ["a", "b", "c"]
    /// ```
    #[cfg(not(feature = "no_index"))]
    #[rhai_fn(pure)]
    pub fn keys(map: &mut Map) -> Array {
        if map.is_empty() {
            Array::new()
        } else {
            map.keys().cloned().map(Into::into).collect()
        }
    }
    /// Return an array with all the property values in the object map.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let m = #{a:1, b:2, c:3};
    ///
    /// print(m.values());      // prints "[1, 2, 3]""
    /// ```
    #[cfg(not(feature = "no_index"))]
    #[rhai_fn(pure)]
    pub fn values(map: &mut Map) -> Array {
        if map.is_empty() {
            Array::new()
        } else {
            map.values().cloned().collect()
        }
    }
}
