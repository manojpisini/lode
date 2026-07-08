use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{LodeError, Result};
use crate::ValidatedRoot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig {
    pub enforce_headers: bool,
    pub default_license: String,
}

impl Default for LicenseConfig {
    fn default() -> Self {
        Self {
            enforce_headers: true,
            default_license: "MIT".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseEntry {
    pub id: String,
    pub name: String,
    pub spdx_id: String,
    pub copyleft: bool,
    pub osi: bool,
    pub fsf: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseMismatch {
    pub path: PathBuf,
    pub expected: String,
    pub found: String,
}

pub fn embedded_licenses() -> Vec<LicenseEntry> {
    vec![
        LicenseEntry {
            id: "mit".into(),
            name: "MIT License".into(),
            spdx_id: "MIT".into(),
            copyleft: false,
            osi: true,
            fsf: true,
        },
        LicenseEntry {
            id: "apache-2.0".into(),
            name: "Apache License 2.0".into(),
            spdx_id: "Apache-2.0".into(),
            copyleft: false,
            osi: true,
            fsf: true,
        },
        LicenseEntry {
            id: "mit-or-apache-2.0".into(),
            name: "MIT OR Apache-2.0".into(),
            spdx_id: "MIT OR Apache-2.0".into(),
            copyleft: false,
            osi: true,
            fsf: true,
        },
        LicenseEntry {
            id: "bsd-3-clause".into(),
            name: "BSD 3-Clause License".into(),
            spdx_id: "BSD-3-Clause".into(),
            copyleft: false,
            osi: true,
            fsf: true,
        },
        LicenseEntry {
            id: "isc".into(),
            name: "ISC License".into(),
            spdx_id: "ISC".into(),
            copyleft: false,
            osi: true,
            fsf: true,
        },
        LicenseEntry {
            id: "gpl-3.0-only".into(),
            name: "GNU General Public License v3.0 only".into(),
            spdx_id: "GPL-3.0-only".into(),
            copyleft: true,
            osi: true,
            fsf: true,
        },
        LicenseEntry {
            id: "mpl-2.0".into(),
            name: "Mozilla Public License 2.0".into(),
            spdx_id: "MPL-2.0".into(),
            copyleft: true,
            osi: true,
            fsf: true,
        },
        LicenseEntry {
            id: "unlicense".into(),
            name: "The Unlicense".into(),
            spdx_id: "Unlicense".into(),
            copyleft: false,
            osi: true,
            fsf: true,
        },
    ]
}

pub fn license_text(id: &str, author: &str, year: &str) -> Result<String> {
    match id {
        "mit" => Ok(format!(
            "MIT License\n\nCopyright (c) {year} {author}\n\n\
             Permission is hereby granted, free of charge, to any person obtaining a copy \
             of this software and associated documentation files (the \"Software\"), to deal \
             in the Software without restriction, including without limitation the rights \
             to use, copy, modify, merge, publish, distribute, sublicense, and/or sell \
             copies of the Software, and to permit persons to whom the Software is \
             furnished to do so, subject to the following conditions:\n\n\
             The above copyright notice and this permission notice shall be included in all \
             copies or substantial portions of the Software.\n\n\
             THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR \
             IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, \
             FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE \
             AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER \
             LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, \
             OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE \
             SOFTWARE."
        )),
        "apache-2.0" => Ok(format!(
            "Apache License\nVersion 2.0, January 2004\nhttp://www.apache.org/licenses/\n\n\
             Copyright {year} {author}\n\n\
             Licensed under the Apache License, Version 2.0 (the \"License\");\n\
             you may not use this file except in compliance with the License.\n\
             You may obtain a copy of the License at\n\n\
             \thttp://www.apache.org/licenses/LICENSE-2.0\n\n\
             Unless required by applicable law or agreed to in writing, software\n\
             distributed under the License is distributed on an \"AS IS\" BASIS,\n\
             WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.\n\
             See the License for the specific language governing permissions and\n\
             limitations under the License."
        )),
        "mit-or-apache-2.0" => Ok(format!(
            "Licensed under either of:\n\n\
             \tMIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)\n\
             \tApache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)\n\n\
             at your option.\n\n\
             Copyright {year} {author}\n\n\
             Unless required by applicable law or agreed to in writing, software\n\
             distributed under the License is distributed on an \"AS IS\" BASIS,\n\
             WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.\n\
             See the License for the specific language governing permissions and\n\
             limitations under the License."
        )),
        "bsd-3-clause" => Ok(format!(
            "BSD 3-Clause License\n\nCopyright (c) {year} {author}\n\n\
             Redistribution and use in source and binary forms, with or without\n\
             modification, are permitted provided that the following conditions are met:\n\n\
             1. Redistributions of source code must retain the above copyright notice, this\n\
             \tlist of conditions and the following disclaimer.\n\n\
             2. Redistributions in binary form must reproduce the above copyright notice,\n\
             \tthis list of conditions and the following disclaimer in the documentation\n\
             \tand/or other materials provided with the distribution.\n\n\
             3. Neither the name of the copyright holder nor the names of its\n\
             \tcontributors may be used to endorse or promote products derived from\n\
             \tthis software without specific prior written permission.\n\n\
             THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS \"AS IS\"\n\
             AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE\n\
             IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE\n\
             DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE\n\
             FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL\n\
             DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR\n\
             SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER\n\
             CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,\n\
             OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE\n\
             OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE."
        )),
        "isc" => Ok(format!(
            "ISC License\n\nCopyright (c) {year} {author}\n\n\
             Permission to use, copy, modify, and/or distribute this software for any\n\
             purpose with or without fee is hereby granted, provided that the above\n\
             copyright notice and this permission notice appear in all copies.\n\n\
             THE SOFTWARE IS PROVIDED \"AS IS\" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH\n\
             REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY\n\
             AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,\n\
             INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM\n\
             LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR\n\
             OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR\n\
             PERFORMANCE OF THIS SOFTWARE."
        )),
        "gpl-3.0-only" => Ok(format!(
            "GNU GENERAL PUBLIC LICENSE\nVersion 3, 29 June 2007\n\n\
             Copyright (c) {year} {author}\n\n\
             This program is free software: you can redistribute it and/or modify\n\
             it under the terms of the GNU General Public License as published by\n\
             the Free Software Foundation, either version 3 of the License, or\n\
             (at your option) any later version.\n\n\
             This program is distributed in the hope that it will be useful,\n\
             but WITHOUT ANY WARRANTY; without even the implied warranty of\n\
             MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the\n\
             GNU General Public License for more details.\n\n\
             You should have received a copy of the GNU General Public License\n\
             along with this program. If not, see <https://www.gnu.org/licenses/>."
        )),
        "mpl-2.0" => Ok(format!(
            "Mozilla Public License Version 2.0\n\n\
             Copyright (c) {year} {author}\n\n\
             1. Definitions\n\n\
             1.1. Contributor\n\
             \tmeans each individual or legal entity that creates, contributes to\n\
             \tthe creation of, or owns Covered Software.\n\n\
             1.2. Contributor Version\n\
             \tmeans the combination of the Contributions of others used by a Contributor\n\
             \tand that particular Contributor's Contribution.\n\n\
             3. Source Code License\n\n\
             3.1. The Initial Developer Grant\n\
             \tThe Initial Developer hereby grants You a world-wide, royalty-free,\n\
             \tnon-exclusive license to use, reproduce, modify, perform, display,\n\
             \tand distribute the Covered Software in Source and Object Form.\n\n\
             See the full license text at https://www.mozilla.org/en-US/MPL/2.0/"
        )),
        "unlicense" => Ok(format!(
            "This is free and unencumbered software released into the public domain.\n\n\
             Anyone is free to copy, modify, publish, use, compile, sell, or\n\
             distribute this software, either in source code form or as a compiled\n\
             binary, for any purpose, commercial or non-commercial, and by any\n\
             means.\n\n\
             In jurisdictions that recognize copyright laws, the author or authors\n\
             of this software dedicate any and all copyright interest in the\n\
             software to the public domain. We make this dedication for the benefit\n\
             of the public at large and to the detriment of our heirs and\n\
             successors. We intend this dedication to be an overt act of\n\
             relinquishment in perpetuity of all present and future rights to this\n\
             software under copyright law.\n\n\
             THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND,\n\
             EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF\n\
             MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.\n\
             IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR\n\
             OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,\n\
             ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE\n\
             OR OTHER DEALINGS IN THE SOFTWARE.\n\n\
             For more information, please refer to <http://unlicense.org/>"
        )),
        _ => Err(LodeError::Message(format!(
            "unknown license id: {id}"
        ))),
    }
}

pub fn apply_license(project_dir: &Path, config: &LicenseConfig, author: &str) -> Result<()> {
    let year = current_year();
    let license_id = config.default_license.to_ascii_lowercase();
    let text = license_text(&license_id, author, &year)?;
    ValidatedRoot::new(project_dir)?.write_atomic("LICENSE", text)?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct ConsistencyReport {
    pub mismatches: Vec<LicenseMismatch>,
}

pub fn check_license_consistency(
    project_dir: &Path,
    config: &LicenseConfig,
) -> Result<ConsistencyReport> {
    let mut mismatches = Vec::new();

    if !config.enforce_headers {
        return Ok(ConsistencyReport { mismatches });
    }

    let header_pattern = format!("License: {}", config.default_license);

    for entry in fs::read_dir(project_dir).map_err(|source| LodeError::Io {
        path: project_dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: project_dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !matches!(ext, "rs" | "toml" | "yml" | "yaml" | "json" | "md" | "txt") {
            continue;
        }

        let content = fs::read_to_string(&path).map_err(|source| LodeError::Io {
            path: path.clone(),
            source,
        })?;
        if content.contains(&header_pattern) {
            continue;
        }

        if content.contains("License:") || content.contains("license:") {
            let found = content
                .lines()
                .find(|l| l.contains("License:") || l.contains("license:"))
                .unwrap_or("unknown")
                .to_string();
            mismatches.push(LicenseMismatch {
                path,
                expected: config.default_license.clone(),
                found,
            });
        }
    }

    Ok(ConsistencyReport { mismatches })
}

fn current_year() -> String {
    "2026".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_licenses_returns_eight() {
        let licenses = embedded_licenses();
        assert_eq!(licenses.len(), 8);
    }

    #[test]
    fn license_text_mit_contains_year_and_author() {
        let text = license_text("mit", "Alice", "2026").unwrap();
        assert!(text.contains("2026"));
        assert!(text.contains("Alice"));
        assert!(text.contains("MIT License"));
    }

    #[test]
    fn license_text_unknown_returns_error() {
        assert!(license_text("nonexistent", "x", "2026").is_err());
    }

    #[test]
    fn apply_license_creates_file() {
        let temp = tempfile::tempdir().unwrap();
        let config = LicenseConfig::default();
        apply_license(temp.path(), &config, "Bob").unwrap();
        let content = fs::read_to_string(temp.path().join("LICENSE")).unwrap();
        assert!(content.contains("Bob"));
    }
}
