use crate::domain::SubscriberEmail;
#[allow(unused)]
pub struct EmailClient {
    sender: SubscriberEmail,
}
#[allow(unused)]
impl EmailClient {
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
