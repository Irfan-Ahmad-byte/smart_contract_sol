use std::fmt;

#[allow(dead_code, unused)]
#[derive(Debug, Clone)]
pub enum WebhookEvent {
    InvoiceCreated,
    DepositSuccessful,
    DepositCreated,
    ConfirmationsPending,
}

impl fmt::Display for WebhookEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let event_str = format!("{self:?}");
        write!(f, "{event_str}")
    }
}
