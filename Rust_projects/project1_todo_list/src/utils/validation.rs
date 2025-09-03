use regex::Regex;
use std::collections::HashSet;

/// Максимальная длина текста задачи
const MAX_TASK_LENGTH: usize = 500;
/// Максимальная длина сообщения
const MAX_MESSAGE_LENGTH: usize = 4000;
/// Минимальная длина текста задачи
const MIN_TASK_LENGTH: usize = 1;

/// Разрешенные символы для текста задач
const ALLOWED_CHARS: &str = "[a-zA-Zа-яА-Я0-9\\s.,!?\\-_()\\[\\]{}@#$%^&*+=|\\\\/:;\"'<>~`]";

/// Запрещенные слова и фразы (базовый список)
const FORBIDDEN_WORDS: &[&str] = &[
    // Пока оставляем пустым - можно добавить специфичные запрещенные слова при необходимости
];

/// Результат валидации
#[derive(Debug, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

/// Валидатор для текста задач
pub struct TaskValidator {
    allowed_chars_regex: Regex,
    forbidden_words: HashSet<String>,
}

impl TaskValidator {
    pub fn new() -> Result<Self, regex::Error> {
        let allowed_chars_regex = Regex::new(ALLOWED_CHARS)?;
        let forbidden_words = FORBIDDEN_WORDS
            .iter()
            .map(|s| s.to_lowercase())
            .collect();

        Ok(Self {
            allowed_chars_regex,
            forbidden_words,
        })
    }

    /// Валидирует текст задачи
    pub fn validate_task_text(&self, text: &str) -> ValidationResult {
        // Проверка длины
        if text.len() < MIN_TASK_LENGTH {
            return ValidationResult::Invalid("Текст задачи не может быть пустым".to_string());
        }

        if text.len() > MAX_TASK_LENGTH {
            return ValidationResult::Invalid(format!(
                "Текст задачи слишком длинный (максимум {} символов)",
                MAX_TASK_LENGTH
            ));
        }

        // Проверка на пустоту после обрезки пробелов
        if text.trim().is_empty() {
            return ValidationResult::Invalid("Текст задачи не может содержать только пробелы".to_string());
        }

        // Проверка разрешенных символов
        if !self.allowed_chars_regex.is_match(text) {
            return ValidationResult::Invalid("Текст содержит недопустимые символы".to_string());
        }

        // Проверка на запрещенные слова
        let text_lower = text.to_lowercase();
        for forbidden_word in &self.forbidden_words {
            if text_lower.contains(forbidden_word) {
                return ValidationResult::Invalid(format!(
                    "Текст содержит запрещенное слово: {}",
                    forbidden_word
                ));
            }
        }

        // Проверка на повторяющиеся символы (защита от спама)
        if self.has_excessive_repetition(text) {
            return ValidationResult::Invalid("Текст содержит слишком много повторяющихся символов".to_string());
        }

        // Проверка на подозрительные паттерны
        if self.has_suspicious_patterns(text) {
            return ValidationResult::Invalid("Текст содержит подозрительные паттерны".to_string());
        }

        ValidationResult::Valid
    }

    /// Санитизирует текст задачи (удаляет опасные символы)
    pub fn sanitize_task_text(&self, text: &str) -> String {
        text.chars()
            .filter(|c| self.allowed_chars_regex.is_match(&c.to_string()))
            .take(MAX_TASK_LENGTH)
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Валидирует сообщение пользователя
    pub fn validate_message(&self, message: &str) -> ValidationResult {
        if message.len() > MAX_MESSAGE_LENGTH {
            return ValidationResult::Invalid(format!(
                "Сообщение слишком длинное (максимум {} символов)",
                MAX_MESSAGE_LENGTH
            ));
        }

        ValidationResult::Valid
    }

    /// Проверяет, есть ли чрезмерное повторение символов
    fn has_excessive_repetition(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let mut current_char = ' ';
        let mut count = 0;
        let max_repetition = 5; // Максимум 5 одинаковых символов подряд

        for &ch in &chars {
            if ch == current_char {
                count += 1;
                if count > max_repetition {
                    return true;
                }
            } else {
                current_char = ch;
                count = 1;
            }
        }

        false
    }

    /// Проверяет на подозрительные паттерны
    fn has_suspicious_patterns(&self, text: &str) -> bool {
        // Проверка на слишком много цифр (более 50% от текста)
        let digit_count = text.chars().filter(|c| c.is_ascii_digit()).count();
        if digit_count > text.len() / 2 {
            return true;
        }

        false
    }
}

/// Валидатор для индексов задач
pub struct TaskIndexValidator;

impl TaskIndexValidator {
    /// Валидирует индекс задачи
    pub fn validate_task_index(index: usize, total_tasks: usize) -> ValidationResult {
        if total_tasks == 0 {
            return ValidationResult::Invalid("Нет доступных задач".to_string());
        }

        if index >= total_tasks {
            return ValidationResult::Invalid(format!(
                "Неверный номер задачи. Доступно задач: {}",
                total_tasks
            ));
        }

        ValidationResult::Valid
    }
}

/// Валидатор для дней месяца (для напоминаний)
pub struct DayValidator;

impl DayValidator {
    /// Валидирует день месяца
    pub fn validate_day(day: u32) -> ValidationResult {
        if day < 1 || day > 31 {
            return ValidationResult::Invalid("День должен быть от 1 до 31".to_string());
        }

        ValidationResult::Valid
    }

    /// Валидирует диапазон дней
    pub fn validate_day_range(start_day: u32, end_day: u32) -> ValidationResult {
        if let ValidationResult::Invalid(msg) = Self::validate_day(start_day) {
            return ValidationResult::Invalid(format!("Начальный день: {}", msg));
        }

        if let ValidationResult::Invalid(msg) = Self::validate_day(end_day) {
            return ValidationResult::Invalid(format!("Конечный день: {}", msg));
        }

        if start_day > end_day {
            return ValidationResult::Invalid("Начальный день не может быть больше конечного".to_string());
        }

        ValidationResult::Valid
    }
}

/// Валидатор для Chat ID
pub struct ChatIdValidator;

impl ChatIdValidator {
    /// Валидирует Chat ID
    pub fn validate_chat_id(chat_id: i64) -> ValidationResult {
        if chat_id <= 0 {
            return ValidationResult::Invalid("Неверный Chat ID".to_string());
        }

        // Проверка на разумные пределы (Telegram Chat ID обычно в определенном диапазоне)
        if chat_id > 999999999999 {
            return ValidationResult::Invalid("Chat ID слишком большой".to_string());
        }

        ValidationResult::Valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_validation() {
        let validator = TaskValidator::new().unwrap();

        // Валидные задачи
        assert_eq!(validator.validate_task_text("Купить молоко"), ValidationResult::Valid);
        assert_eq!(validator.validate_task_text("Позвонить маме"), ValidationResult::Valid);
        assert_eq!(validator.validate_task_text("Задача с цифрами 123"), ValidationResult::Valid);

        // Невалидные задачи
        assert!(matches!(validator.validate_task_text(""), ValidationResult::Invalid(_)));
        assert!(matches!(validator.validate_task_text("   "), ValidationResult::Invalid(_)));
        assert!(matches!(validator.validate_task_text(&"a".repeat(501)), ValidationResult::Invalid(_)));
    }

    #[test]
    fn test_forbidden_words() {
        let validator = TaskValidator::new().unwrap();
        
        // Теперь эти задачи должны быть валидными
        assert_eq!(validator.validate_task_text("Купить bitcoin"), ValidationResult::Valid);
        assert_eq!(validator.validate_task_text("Перейти на http://example.com"), ValidationResult::Valid);
    }

    #[test]
    fn test_excessive_repetition() {
        let validator = TaskValidator::new().unwrap();
        
        assert!(matches!(validator.validate_task_text("Купить молоко!!!!!!"), ValidationResult::Invalid(_)));
        assert!(matches!(validator.validate_task_text("Задача      с      пробелами"), ValidationResult::Invalid(_)));
    }

    #[test]
    fn test_task_index_validation() {
        assert_eq!(TaskIndexValidator::validate_task_index(0, 3), ValidationResult::Valid);
        assert_eq!(TaskIndexValidator::validate_task_index(2, 3), ValidationResult::Valid);
        assert!(matches!(TaskIndexValidator::validate_task_index(3, 3), ValidationResult::Invalid(_)));
        assert!(matches!(TaskIndexValidator::validate_task_index(0, 0), ValidationResult::Invalid(_)));
    }

    #[test]
    fn test_day_validation() {
        assert_eq!(DayValidator::validate_day(1), ValidationResult::Valid);
        assert_eq!(DayValidator::validate_day(31), ValidationResult::Valid);
        assert!(matches!(DayValidator::validate_day(0), ValidationResult::Invalid(_)));
        assert!(matches!(DayValidator::validate_day(32), ValidationResult::Invalid(_)));
    }
}
