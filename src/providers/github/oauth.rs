use pyo3::prelude::*;
use pyo3::types::PyString;
use reqwest::Client;

#[pyclass]
pub struct GitHubOAuthClient;

#[pymethods]
impl GitHubOAuthClient {
    #[staticmethod]
    #[pyo3(name = "exchange_code_for_token")]
    pub fn exchange_code_for_token_py<'py>(
        py: Python<'py>,
        client_id: String,
        client_secret: String,
        code: String,
        redirect_uri: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let client = Client::new();
            let params = [
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ];
            let res = client
                .post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .form(&params)
                .send()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            let json: serde_json::Value = res
                .json()
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            Python::with_gil(|py| -> PyResult<PyObject> {
                if let Some(token) = json.get("access_token").and_then(|v| v.as_str()) {
                    // Convert PyString to PyAny before unbinding
                    let py_str = PyString::new(py, token).into_any().unbind();
                    Ok(py_str)
                } else {
                    Err(pyo3::exceptions::PyRuntimeError::new_err(
                        json.get("error_description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error")
                            .to_string(),
                    ))
                }
            })
        })
    }
}
