pub trait IntoCliArgsVec {
    fn into_prefixed_cmd_arg(
        &self,
        long_option: String,
        value_prefix: Option<String>,
    ) -> Vec<String>;

    fn into_cmd_arg(&self, long_option: &'static str) -> Vec<String> {
        self.into_prefixed_cmd_arg(format!("--{}", long_option), None)
    }

    fn into_docker_build_arg(&self, build_arg_name: &'static str) -> Vec<String> {
        self.into_prefixed_cmd_arg("--build-arg".into(), Some(format!("{}=", build_arg_name)))
    }
}

impl IntoCliArgsVec for String {
    fn into_prefixed_cmd_arg(
        &self,
        long_option: String,
        value_prefix: Option<String>,
    ) -> Vec<String> {
        vec![
            long_option,
            value_prefix.map_or(self.clone(), |prefix| format!("{}{}", prefix, self)),
        ]
    }
}

impl IntoCliArgsVec for &str {
    fn into_prefixed_cmd_arg(
        &self,
        long_option: String,
        value_prefix: Option<String>,
    ) -> Vec<String> {
        vec![
            long_option,
            value_prefix.map_or(self.to_string(), |prefix| format!("{}{}", prefix, self)),
        ]
    }
}
