use crate::{configuration::EmailClientSettings, domain::SubscriberEmail};
use lettre::{
    transport::smtp::{authentication::Credentials, PoolConfig},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use secrecy::ExposeSecret;
use std::time::Duration;
#[derive(Clone)]
pub struct EmailClient {
    sender: SubscriberEmail,
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
}
#[allow(unused)]
impl EmailClient {
    pub fn new(email_config: EmailClientSettings) -> Result<Self, Box<dyn std::error::Error>> {
        let sender = email_config
            .sender()
            .expect("Invalid sender email address.");
        let smtp_credentials = Credentials::new(
            email_config.username,
            email_config.password.expose_secret().to_string(),
        );
        println!("Making connection to SMTP");
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&email_config.base_url)?
                .credentials(smtp_credentials)
                .pool_config(
                    PoolConfig::new()
                        .min_idle(3)
                        .max_size(10)
                        .idle_timeout(Duration::new(300, 0)),
                )
                .build();

        println!("SMTP connection created succuessfully");
        Ok(Self { sender, mailer })
    }

    pub async fn send_email_smtp(
        &self,
        // mailer: &AsyncSmtpTransport<Tokio1Executor>,
        to: &str,
        subject: &str,
        body: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("sdsd{:?}", self.mailer);
        let sender = self.sender.as_ref();
        let email = Message::builder()
            .from(self.sender.as_ref().parse()?)
            .to(to.parse()?)
            .subject(subject)
            .body(body.to_string())?;

        println!("Sending Email");
        self.mailer.send(email).await?;
        println!("Mail Send Successfully");
        Ok(())
    }
}
