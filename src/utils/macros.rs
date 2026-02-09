#[macro_export]
macro_rules! impl_string_value_object {
    ($t:ty) => {
        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl AsRef<str> for $t {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}
