use crate::open_string_enum::open_string_enum;

open_string_enum! {
    pub enum TwoFactorMethod {
        EmailOtp => "emailOtp",
        Otp => "otp",
        Totp => "totp",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::TwoFactorMethod;

    #[test]
    fn two_factor_methods_preserve_unknown_wire_values() {
        let known: TwoFactorMethod = serde_json::from_value(json!("totp")).unwrap();
        let unknown: TwoFactorMethod = serde_json::from_value(json!("futureMethod")).unwrap();

        assert_eq!(known, TwoFactorMethod::Totp);
        assert_eq!(unknown.as_str(), "futureMethod");
        assert_eq!(
            serde_json::to_value(unknown).unwrap(),
            json!("futureMethod")
        );
    }
}
