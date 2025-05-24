/*
    lib.rs

    Provides functions for interacting with the Python API.
*/

pub mod io;
pub mod norm;
pub mod set;

use pyo3::{prelude::*, types::PyString};
use rayon::ThreadPool;
use crate::norm::{_normalize_text as _inner_normalize_text, normalize_owned_value};
use crate::io::{ArchiveWriter, ArchiveReader};
use simd_json::OwnedValue;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;

#[pyfunction]
// Mangling the name so the python side can have neat docstrings
fn __normalize_text(
    text: &Bound<'_, PyString>,
) -> PyResult<String> {
    // Convert the text to a string
    let text = text.to_string_lossy();

    _inner_normalize_text(&text)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Normalization error: {}", e)))
        .map(|s| s.to_string())
}

#[pyfunction]
fn __normalize_jsonl_file(
    input_file: &str,
    output_file: &str,
    text_column: &str,
    workers: usize,
) -> PyResult<()> {

    // Release the GIL for the duration of the heavy IO/CPU work
    Python::with_gil(|py| {
        py.allow_threads(|| {
            let input_path = std::path::Path::new(input_file);
            let output_path = std::path::Path::new(output_file);

            let reader = ArchiveReader::new(
                &input_path,
            ).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create ArchiveReader: {}", e)))?;

            let processed : Vec<OwnedValue> = reader.filter_map(|x| x.ok()).collect();

            let pool : ThreadPool = ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create thread pool: {}", e)))?;

            let normalized: Vec<OwnedValue> = pool.install(|| {
                processed.into_par_iter().map(|mut val: OwnedValue| {
                    // Normalize the text in the JSON object
                    normalize_owned_value(&mut val, text_column)
                        .expect("Failed to normalize text");
                    // Print the normalized JSON object
                    val
                }).collect::<Vec<_>>()
            });

            let mut writer = ArchiveWriter::new(
                &output_path,
            ).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create ArchiveWriter: {}", e)))?;

            for val in normalized.into_iter() {
                // Write the normalized JSON object to the output file
                writer.write(&val)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to write JSON object: {}", e)))?;
            }
            writer.close()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to close ArchiveWriter: {}", e)))?;
            Ok::<(), PyErr>(())
        })
    })?;
    Ok::<(), PyErr>(())
}


#[pymodule]
fn sstn(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add the normalize_text function to the module
    m.add_function(wrap_pyfunction!(__normalize_text, m)?)?;
    m.add_function(wrap_pyfunction!(__normalize_jsonl_file, m)?)?;
    Ok(())
}
