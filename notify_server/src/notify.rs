use chat_core::{Chat, Message};

// PERFORM pg_notify('chat_updated', json_build_object('op', TG_OP,'old', OLD, 'new', NEW )::text);
#[allow(dead_code)]
pub struct ChatUpdated {
    pub op: String,
    pub old: Option<Chat>,
    pub new: Option<Chat>,
}

// PERFORM pg_notify('chat_message_created', row_to_json(NEW)::text);
#[allow(dead_code)]

pub struct ChatMessageCreated {
    pub new: Message,
}
