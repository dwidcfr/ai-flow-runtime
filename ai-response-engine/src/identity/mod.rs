use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub name: String,
    #[serde(default)]
    pub bank_name: String,
    pub role: String,
    pub communication_style: String,
    pub language: String,
    #[serde(default)]
    pub tone: String,
    pub constraints: Vec<String>,
    pub conversation_rules: Vec<String>,
    pub additional_instructions: String,
}

impl AgentIdentity {
    pub fn default_payment_agent() -> Self {
        Self {
            name: "Alex".to_string(),
            bank_name: "Acme Insurance".to_string(),
            role: "Оператор по взысканию задолженности".to_string(),
            communication_style: "Вежливый, профессиональный, краткий".to_string(),
            language: "ru".to_string(),
            tone: "Спокойный, уверенный".to_string(),
            constraints: vec![
                "Не обещать скидки без подтверждения".to_string(),
                "Не раскрывать внутренние процессы компании".to_string(),
            ],
            conversation_rules: vec![
                "Обращаться к клиенту по имени, если оно известно".to_string(),
                "Отвечать только на основе переданного контекста".to_string(),
            ],
            additional_instructions: "Помогать клиенту завершить оплату или уточнить детали."
                .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_payment_agent_has_required_fields() {
        let identity = AgentIdentity::default_payment_agent();
        assert_eq!(identity.name, "Alex");
        assert_eq!(identity.bank_name, "Acme Insurance");
        assert!(!identity.role.is_empty());
        assert_eq!(identity.language, "ru");
        assert!(!identity.constraints.is_empty());
        assert!(!identity.conversation_rules.is_empty());
    }
}
