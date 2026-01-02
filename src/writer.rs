use crate::degrees::{analyze_degree, degrees};
use itertools::Itertools;
use log::error;
use std::{
    fs::{create_dir, write},
    path::Path,
};

fn write_folder(output_dir: &Path) {
    if !output_dir.exists()
        && let Err(e) = create_dir(output_dir)
    {
        panic!("Output dir creation: {e}")
    }
}

pub fn write_all() {
    let output_dir = Path::new("output");
    write_folder(output_dir);
    let index = r#"= Unified Course Descriptions for Some UNIBO Degrees

https://cartabinaria.students.cs.unibo.it/en/wiki/web-scraper/course-description-merged/[Documentation]

"#.to_owned() + degrees().unwrap().iter().map(|d| {
        let slug = &d.slug;
        let output_file = output_dir.join(format!("degree-{slug}.adoc"));
        write(output_file, analyze_degree(d).unwrap()).unwrap();
        format!(r#"== {}

xref:degree-{}.adoc[web] | link:degree-{}.pdf[PDF] | link:degree-{}.adoc[Asciidoc]

"#,
                d.name, d.slug, d.slug, d.slug
            )
        }).join("\n").as_str();
    if let Err(e) = write(output_dir.join("index.adoc"), index) {
        error!("Could not write index: {e}")
    };
}
