pub fn emit(function: &idlc_mir::Function) -> String {
    function
        .doc
        .as_ref()
        .map(|doc| {
            let lines = doc.lines();
            let mut doc = String::new();
            for line in lines {
                let line = line.trim_start();
                let docstring = if line.starts_with('*') {
                    line.split_once('*').unwrap().1
                } else {
                    line
                };

                doc += "/// ";
                doc += docstring.trim();
                doc += "\n"
            }
            doc
        })
        .unwrap_or_default()
}
