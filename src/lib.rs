//! A crate that allows you to use `externref`s with your Wasm modules.
#![forbid(missing_docs)]

/// A struct acting as a Rust interpretation of an `externref` that will get modified after compile
/// time. Because Rust itself doesn't have a concept of `externref` we need to transform the output
/// wasm module after compilating to match it's import/export usages.
///
/// Example:
///
/// ```rust,ignore
/// #[externref(module_name = "module", import_name = "intoRef")]
/// extern "C" fn into_ref(value: u32) -> ExternRef;
///
/// #[externref(module_name = "module", import_name = "fromRef")]
/// extern "C" fn from_ref(externref: ExternRef) -> u32;
///
/// const VALUE: u32 = 100;
/// let reffed: ExternRef = into_ref(VALUE);
/// let unreffed = from_ref(reffed);
///
/// assert_eq!(unreffed, VALUE);
/// ```
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct ExternRef {
    inner: usize,
}

#[cfg(target_arch = "wasm32")]
impl ExternRef {
    /// Creates a new [ExternRef] with the value of `null`.
    pub fn null() -> Self {
        // TODO(zeb): Should we call a function that'll have it's definition swapped at
        // transform time that just executes `ref.null`?
        todo!("cannot call ref.null instruction until module is transformed");
    }

    /// Checks if this ref is null.
    pub fn is_null(&self) -> bool {
        // TODO(zeb): Should we call a function that'll have it's definition swapped at
        // transform time that just executes `ref.is_null`?
        todo!("cannot call ref.null instruction until module is transformed");
    }

    /// Converts a [usize] into a [ExternRef].
    ///
    /// # Safety
    /// It is possible to run into undefined behavior if the raw reference is not an extern ref
    /// from the host.
    pub unsafe fn from_usize(raw_ref: usize) -> Self {
        Self { inner: raw_ref }
    }
}

impl From<ExternRef> for usize {
    fn from(val: ExternRef) -> Self {
        val.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_alignment() {
        assert_eq!(
            core::mem::align_of::<ExternRef>(),
            core::mem::align_of::<usize>()
        )
    }

    #[test]
    fn same_layout() {
        assert_eq!(
            core::mem::align_of::<ExternRef>(),
            core::mem::align_of::<usize>()
        )
    }
}
