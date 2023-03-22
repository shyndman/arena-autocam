use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::Result;
use base64::Engine as _;
use hmac::{Hmac, Mac};
use http::header;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::blocking::{ClientBuilder, RequestBuilder};
use reqwest::redirect::Policy;
use reqwest::{IntoUrl, Method, Url};
use sha2::Sha256;

use super::models::{AssemblyDefinition, DocumentElement};

const BASE_URL: &str = "https://cad.onshape.com/api/v5";

type HmacSha256 = Hmac<Sha256>;

pub struct OnShapeClient {
    pub http_client: reqwest::blocking::Client,
    pub access_key: String,
    pub secret_key: String,
}

impl OnShapeClient {
    pub fn new(access_key: String, secret_key: String) -> Result<Self> {
        Ok(Self {
            http_client: ClientBuilder::new()
                .gzip(true)
                .redirect(Policy::none())
                .build()?,
            access_key: access_key,
            secret_key: secret_key,
        })
    }

    pub fn get_document_elements(
        &self,
        doc_id: &String,
        workspace_id: &String,
    ) -> Result<HashMap<String, DocumentElement>> {
        let url = format!(
            "{}/documents/d/{document_id}/w/{workspace_id}/elements",
            BASE_URL,
            document_id = doc_id,
            workspace_id = workspace_id
        );
        let elements: Vec<DocumentElement> = self.request(Method::GET, url).send()?.json()?;

        let mut elements_by_id = HashMap::new();
        for e in elements {
            elements_by_id.insert(e.id.clone(), e);
        }
        Ok(elements_by_id)
    }

    pub fn get_assembly(
        &self,
        doc_id: &String,
        workspace_id: &String,
        assembly_id: &String,
    ) -> Result<AssemblyDefinition> {
        let url = format!(
            "{}/assemblies/d/{document_id}/w/{workspace_id}/e/{assembly_id}",
            BASE_URL,
            document_id = doc_id,
            workspace_id = workspace_id,
            assembly_id = assembly_id,
        );

        Ok(self.request(Method::GET, url).send()?.json()?)
    }

    pub fn get_part_stl(
        &self,
        doc_id: &String,
        microversion_id: &String,
        element_id: &String,
        part_id: &String,
        configuration: &String,
    ) -> Result<String> {
        use std::str::FromStr;
        let mut url = Url::from_str(&format!(
            "{}/parts/d/{document_id}/m/{microversion_id}/e/{element_id}/partid/{part_id}/stl?",
            BASE_URL,
            document_id = doc_id,
            microversion_id = microversion_id,
            element_id = element_id,
            part_id = part_id,
        ))?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("mode", "text");
            query.append_pair("units", "millimeter");
            query.append_pair("angleTolerance", "0.04363323129985824");
            query.append_pair("chordTolerance", "0.06");
            query.append_pair("minFacetWidth", "0.025");
            query.append_pair("configuration", configuration);
        }

        let res = self.request(Method::GET, url).send()?;
        assert!(res.status().is_redirection(), "Redirect expected");

        let redirect_url = res
            .headers()
            .get("location")
            .expect("Missing location header")
            .to_str()?;
        Ok(self.request(Method::GET, redirect_url).send()?.text()?)
    }

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        let url = url.into_url().expect("Could not convert to URL");
        let content_type = mime::APPLICATION_JSON;

        // Prepare the signature
        let nonce = create_nonce();
        let date = httpdate::fmt_http_date(SystemTime::now());
        let path = url.path();
        let query: String = url.query().map_or("".into(), |val| {
            percent_encoding::percent_decode_str(val)
                .decode_utf8()
                .expect("Error parsing query")
                .into_owned()
        });

        let signature_plaintext =
            // NOTE: While not documented, the trailing newline is a requirement
            format!("{method}\n{nonce}\n{date}\n{content_type}\n{path}\n{query}\n")
                .to_lowercase();

        let mac = {
            let mut m = HmacSha256::new_from_slice(self.secret_key.as_bytes())
                .expect("HMAC can take key of any size");
            m.update(signature_plaintext.as_bytes());
            m
        };

        let authorization_val = format!(
            "On {access_key}:HmacSHA256:{signature}",
            access_key = self.access_key,
            // NOTE: The OnShape API requires that the signature be encoded as base64 with
            // padding characters, and as such, we use the STANDARD engine (not the
            // STANDARD_NO_PAD).
            signature =
                base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
        );

        self.http_client
            .request(method, url)
            .header(header::AUTHORIZATION, authorization_val)
            .header(
                header::ACCEPT,
                "application/vnd.onshape.v2+json;charset=UTF-8;qs=0.2",
            )
            .header(header::CONTENT_TYPE, content_type.to_string())
            .header(header::DATE, date)
            .header("On-Nonce", nonce)
    }
}

fn create_nonce() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(25)
        .map(char::from)
        .collect()
}
