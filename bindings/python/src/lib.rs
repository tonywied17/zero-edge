//! Python bindings for the pamoja SDK, generated with PyO3.
//!
//! This crate is the generated low-level surface (the contract tier): it exposes
//! the Rust core and capability crates to Python one-to-one. A hand-written,
//! idiomatic facade wraps it for everyday use; see the `pamoja` Python package.
//!
//! The native module is imported as `pamoja._core` and re-exported verbatim at
//! `pamoja.raw`.

use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::gen_stub_pyfunction};

#[cfg(feature = "mqtt")]
mod mqtt;

pyo3::create_exception!(
    pamoja,
    PamojaError,
    pyo3::exceptions::PyException,
    "Raised when a pamoja operation fails."
);

/// Returns the version of the native pamoja module.
#[gen_stub_pyfunction]
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// The generated low-level Python surface for the pamoja core.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add("PamojaError", m.py().get_type::<PamojaError>())?;
    #[cfg(feature = "mqtt")]
    {
        m.add_class::<mqtt::MqttClient>()?;
        m.add_class::<mqtt::MqttMessage>()?;
    }
    Ok(())
}

define_stub_info_gatherer!(stub_info);
