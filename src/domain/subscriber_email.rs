use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use utoipa::ToSchema;
use validator::validate_email;

#[derive(Debug, Clone, Deserialize, PartialEq, ToSchema)]
pub struct EmailObject(String);

impl EmailObject {
    pub fn parse(s: String) -> Result<EmailObject, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid email.", s))
        }
    }
    pub fn get<'a>(&'a self) -> &'a str {
        &self.0
    }
}

impl AsRef<str> for EmailObject {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EmailObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub fn deserialize_subscriber_email<'de, D>(deserializer: D) -> Result<EmailObject, D::Error>
where
    D: Deserializer<'de>,
{
    struct SubscriberEmailVisitor;

    impl<'de> Visitor<'de> for SubscriberEmailVisitor {
        type Value = EmailObject;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a valid subscriber email string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            EmailObject::parse(value.to_string()).map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_str(SubscriberEmailVisitor)
}

#[cfg(test)]
mod tests {

    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck_macros::quickcheck;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use crate::domain::EmailObject;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);

            Self(email)
        }
    }
    #[quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        dbg!(&valid_email.0); // `use cargo test valid_emails -- --nocapture` to print the emails on console
        EmailObject::parse(valid_email.0).is_ok()
    }
}
