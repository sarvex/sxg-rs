// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::{Error, Result};
use std::path::Path;
use std::process::Command;

/// Executes a command, and returns the stdout as bytes.
fn execute(command: &mut Command) -> Result<Vec<u8>> {
    let output = command
        .output()
        .map_err(|e| Error::new(e).context("Failed to execute command"))?;
    Ok(output.stdout)
}

/// Executes a command, and parses the stdout as a string.
fn execute_and_parse_stdout(command: &mut Command) -> Result<String> {
    let stdout = execute(command)?;
    String::from_utf8(stdout)
        .map_err(|e| Error::new(e).context("The stdout contains non-utf8 bytes."))
}

/// Writes content into a new file.
/// Returns error if a file already exists.
pub fn write_new_file(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    if path.exists() {
        Err(Error::msg(format!(
            "Cowardly refuse to overwrite {:?}",
            path
        )))
    } else {
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Generates a private key, and returns it in PEM format.
/// Writes PEM to `output_file`.
/// Returns error if `output_file` already exists.
pub fn create_private_key_pem(output_file: impl AsRef<Path>) -> Result<String> {
    let privkey_pem = execute_and_parse_stdout(
        Command::new("openssl")
            .arg("ecparam")
            .arg("-outform")
            .arg("pem")
            .arg("-name")
            .arg("prime256v1")
            .arg("-genkey"),
    )?;
    write_new_file(output_file, &privkey_pem)?;
    Ok(privkey_pem)
}

/// Generates a certificate request, and returns it in PEM format.
/// Writes PEM to `output_file`.
/// Returns error if `output_file` already exists.
pub fn create_certificate_request_pem(
    domain: &str,
    private_key_file: impl AsRef<Path>,
    output_file: impl AsRef<Path>,
) -> Result<String> {
    let cert_csr_pem = execute_and_parse_stdout(
        Command::new("openssl")
            .arg("req")
            .arg("-new")
            .arg("-sha256")
            .arg("-key")
            .arg(private_key_file.as_ref().as_os_str())
            .arg("-subj")
            .arg(format!("/CN={}/O=Test/C=US", domain)),
    )?;
    write_new_file(output_file, &cert_csr_pem)?;
    Ok(cert_csr_pem)
}

/// Create a certificate by signing the certificate request file
/// by the private key,
/// and returns the certificate in PEM format.
/// Writes PEM to `output_file`.
/// Returns error if `output_file` already exists.
pub fn create_certificate(
    private_key_file: impl AsRef<Path>,
    certificiate_request_file: impl AsRef<Path>,
    ext_file: impl AsRef<Path>,
    output_file: impl AsRef<Path>,
) -> Result<String> {
    let cert_pem = execute_and_parse_stdout(
        Command::new("openssl")
            .arg("x509")
            .arg("-req")
            .arg("-days")
            .arg("90")
            .arg("-in")
            .arg(certificiate_request_file.as_ref().as_os_str())
            .arg("-signkey")
            .arg(private_key_file.as_ref().as_os_str())
            .arg("-extfile")
            .arg(ext_file.as_ref().as_os_str()),
    )?;
    write_new_file(output_file, &cert_pem)?;
    Ok(cert_pem)
}

pub fn get_certificate_sha256(certificate_file: impl AsRef<Path>) -> Result<Vec<u8>> {
    let public_key_pem = execute_and_parse_stdout(
        Command::new("openssl")
            .arg("x509")
            .arg("-pubkey")
            .arg("-noout")
            .arg("-in")
            .arg(certificate_file.as_ref().as_os_str()),
    )?;
    let public_key_der = sxg_rs::config::get_der(&public_key_pem, "PUBLIC KEY")?;
    Ok(sxg_rs::utils::get_sha(&public_key_der))
}