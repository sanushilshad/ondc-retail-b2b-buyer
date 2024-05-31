use crate::{configuration::EmailClientSettings, domain::EmailObject};
use async_trait::async_trait;
use lettre::{
    message::SinglePart,
    transport::smtp::{authentication::Credentials, PoolConfig},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use secrecy::ExposeSecret;
use std::time::Duration;
// #[derive(Clone)]
// pub struct EmailClient {
//     sender: EmailObject,
//     pub mailer: AsyncSmtpTransport<Tokio1Executor>,
// }
// impl EmailClient {
//     #[tracing::instrument]
//     pub fn new(email_config: EmailClientSettings) -> Result<Self, Box<dyn std::error::Error>> {
//         let sender = email_config
//             .sender()
//             .expect("Invalid sender email address.");
//         let smtp_credentials = Credentials::new(
//             email_config.username,
//             email_config.password.expose_secret().to_string(),
//         );
//         tracing::info!("Establishing  connection to the SMTP server.");
//         let mailer: AsyncSmtpTransport<Tokio1Executor> =
//             AsyncSmtpTransport::<Tokio1Executor>::relay(&email_config.base_url)?
//                 .credentials(smtp_credentials)
//                 .pool_config(
//                     PoolConfig::new()
//                         .min_idle(3)
//                         .max_size(10)
//                         .idle_timeout(Duration::new(300, 0)),
//                 )
//                 .build();

//         tracing::info!("SMTP connection created succuessfully");
//         Ok(Self { sender, mailer })
//     }

//     pub async fn send_text_email(
//         &self,
//         // mailer: &AsyncSmtpTransport<Tokio1Executor>,
//         to: &str,
//         subject: &str,
//         body: String,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         tracing::info!("SMTP Parameters: {:?}", self.mailer);
//         let email = Message::builder()
//             .from(self.sender.as_ref().parse()?)
//             .to(to.parse()?)
//             .subject(subject)
//             .body(body.to_string())?;

//         tracing::info!("Sending Email");
//         self.mailer.send(email).await?;
//         tracing::info!("Mail Send Successfully");
//         Ok(())
//     }

//     pub async fn send_html_email(
//         &self,
//         // mailer: &AsyncSmtpTransport<Tokio1Executor>,
//         to: &str,
//         subject: &str,
//         body: String,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         tracing::info!("SMTP Parameters: {:?}", self.mailer);
//         Ok(())
//     }
// }

#[async_trait]
pub trait GenericEmailService: Send + Sync {
    async fn send_text_email(
        &self,
        to: &str,
        subject: &str,
        body: String,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn send_html_email(
        &self,
        to: &str,
        subject: &str,
        body: String,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct SmtpEmailClient {
    pub sender: EmailObject,
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpEmailClient {
    #[tracing::instrument]
    pub fn new(email_config: &EmailClientSettings) -> Result<Self, Box<dyn std::error::Error>> {
        let sender = email_config
            .sender()
            .expect("Invalid sender email address.");
        let smtp_credentials = Credentials::new(
            email_config.username.to_string(),
            email_config.password.expose_secret().to_string(),
        );
        tracing::info!("Establishing  connection to the SMTP server.");
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&email_config.base_url)?
                .credentials(smtp_credentials)
                .pool_config(
                    PoolConfig::new()
                        .min_idle(3)
                        .max_size(10)
                        .idle_timeout(Duration::new(30000, 0)),
                )
                .build();

        tracing::info!("SMTP connection created succuessfully");
        Ok(Self { sender, mailer })
    }
}

#[async_trait]
impl GenericEmailService for SmtpEmailClient {
    async fn send_text_email(
        &self,
        to: &str,
        subject: &str,
        body: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("SMTP Parameters: {:?}", self.mailer);
        let email = Message::builder()
            .from(self.sender.as_ref().parse()?)
            .to(to.parse()?)
            .subject(subject)
            .body(body.to_string())?;

        tracing::info!("Sending Email");
        self.mailer.send(email).await?;
        tracing::info!("Mail Send Successfully");
        Ok(())
    }

    async fn send_html_email(
        &self,
        // mailer: &AsyncSmtpTransport<Tokio1Executor>,
        to: &str,
        subject: &str,
        _body: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("SMTP Parameters: {:?}", self.mailer);
        let email = Message::builder()
            .from(self.sender.as_ref().parse()?)
            .to(to.parse()?)
            .subject(subject)
            .singlepart(SinglePart::html(String::from(
                "<p><b>Hello</b>, <i>Grape</i>! <img src=cid:123></p>",
            )))?;

        tracing::info!("Sending HTML Email");
        self.mailer.send(email).await?;
        tracing::info!("HTML Email Sent Successfully");
        Ok(())
    }
}

pub struct DummyEmailClient {}

impl DummyEmailClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("Establishing dummy connection to the SMTP server.");
        tracing::info!("Dummy SMTP connection created succuessfully");
        Ok(Self {})
    }
}

#[async_trait]
impl GenericEmailService for DummyEmailClient {
    async fn send_text_email(
        &self,
        _to: &str,
        _subject: &str,
        _body: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn send_html_email(
        &self,
        _to: &str,
        _subject: &str,
        _body: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
