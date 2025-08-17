pub fn parse_task_list(text: &str) -> Vec<String> {
    let mut tasks = Vec::new();
    
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Удаляем нумерацию если есть (1., 2), 3-, * и т.д.)
        let clean_line = if let Some(pos) = line.find(|c: char| c.is_alphabetic() || c == '"' || c == '(' || c == '[') {
            let prefix = &line[..pos];
            if prefix.chars().all(|c| c.is_numeric() || ".-*) ".contains(c)) {
                line[pos..].trim()
            } else {
                line
            }
        } else {
            line
        };
        
        if !clean_line.is_empty() && clean_line.len() <= 500 {
            tasks.push(clean_line.to_string());
        }
    }
    
    tasks
}