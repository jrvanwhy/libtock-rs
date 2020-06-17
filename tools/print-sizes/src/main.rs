// Architectures that we expect the examples to be built for.
const ARCHITECTURES: [&str; 2] = ["riscv32imc-unknown-none-elf", "thumbv7em-none-eabi"];

// The order of these fields actually matters, because it affects the derived
// Ord impl. I have a suspicion that when I introduce size diffs into the CI,
// this order will make the eventual diffs easier to understand than other
// orderings.
#[derive(Eq, PartialEq, PartialOrd, Ord)]
struct Example {
    name: String,
    arch: &'static str,
    path: std::path::PathBuf,
}

// Finds the example binaries and returns a list of their paths.
fn find_examples() -> Vec<Example> {
    // Find target/ using std::env::current_exe().
    let exe_dir = std::env::current_exe().expect("Unable to find executable location");
    let target_dir = exe_dir
        .parent()
        .expect("Unable to find target/ directory")
        .parent()
        .expect("Unable to find target/ directory");

    let mut examples = Vec::new();

    for arch in &ARCHITECTURES {
        // Set examples_dir to target/$ARCH/examples/
        let mut examples_dir = target_dir.to_path_buf();
        examples_dir.push(arch);
        examples_dir.push("release");
        examples_dir.push("examples");

        // If the architecture's examples directory exists, iterate through the
        // files through it and search for examples. If the directory doesn't
        // exist we skip this architecture.
        if let Ok(read_dir) = examples_dir.read_dir() {
            for file in read_dir.filter_map(Result::ok) {
                use std::os::unix::ffi::OsStrExt;

                // Skip entries that are not files. If file_type() returns
                // Err(_) we skip the entry as well.
                if !file.file_type().map_or(false, |t| t.is_file()) {
                    continue;
                }

                // Skip files with dots (*.d files) and hyphens (-$HASH) in
                // them.
                if file.file_name().as_bytes().contains(&b'.')
                    || file.file_name().as_bytes().contains(&b'-')
                {
                    continue;
                }

                examples.push(Example {
                    name: file.file_name().to_string_lossy().into_owned(),
                    arch,
                    path: file.path(),
                });
            }
        }
    }

    examples
}

struct ElfSizes {
    bss: u64,
    data: u64,
    rodata: u64,
    text: u64,
}

impl std::ops::Add for &ElfSizes {
    type Output = ElfSizes;

    fn add(self, rhs: &ElfSizes) -> ElfSizes {
        ElfSizes {
            bss: self.bss + rhs.bss,
            data: self.data + rhs.data,
            rodata: self.rodata + rhs.rodata,
            text: self.text + rhs.text,
        }
    }
}

fn get_sizes(path: &std::path::Path) -> ElfSizes {
    let file = elf::File::open_path(path).expect("Unable to open example binary");
    let mut sizes = ElfSizes {
        bss: 0,
        data: 0,
        rodata: 0,
        text: 0,
    };
    for section in file.sections {
        match section.shdr.name.as_ref() {
            ".bss" => sizes.bss = section.shdr.size,
            ".data" => sizes.data = section.shdr.size,
            ".rodata" => sizes.rodata = section.shdr.size,
            ".text" => sizes.text = section.shdr.size,
            _ => {}
        }
    }
    sizes
}

struct ExampleData {
    name: String,
    arch: &'static str,
    sizes: ElfSizes,
}

fn main() {
    let mut examples = find_examples();
    examples.sort_unstable();
    let example_data: Vec<_> = examples
        .drain(..)
        .map(|example| ExampleData {
            name: example.name,
            arch: example.arch,
            sizes: get_sizes(&example.path),
        })
        .collect();

    let name_width = 20;
    let arch_width = example_data
        .iter()
        .map(|a| a.arch.len())
        .max()
        .expect("No examples found");
    let section_width = 7;

    // TODO: We do not currently print out .rodata's size. Currently, the linker
    // script embeds .rodata in .text, so we don't see it as a separate section
    // here. We should modify the linker script to put .rodata in its own
    // section. Until that is done, .rodata's size will be counted as part of
    // .text, so we'll just print .text's size for now.
    println!(
        "{0:1$} {2:3$} {4:>7$} {5:>7$} {6:>7$}",
        "Example", name_width, "Architecture", arch_width, ".bss", ".data", ".text", section_width
    );
    for data in &example_data {
        println!(
            "{0:1$} {2:3$} {4:7$} {5:7$} {6:7$}",
            data.name,
            name_width,
            data.arch,
            arch_width,
            data.sizes.bss,
            data.sizes.data,
            data.sizes.text,
            section_width
        );
    }

    for arch in &ARCHITECTURES {
        let mut totals = ElfSizes {
            bss: 0,
            data: 0,
            rodata: 0,
            text: 0,
        };
        for data in example_data.iter().filter(|d| d.arch == *arch) {
            totals = &totals + &data.sizes;
        }
        println!(
            "{0:1$} {2:3$} {4:7$} {5:7$} {6:7$}",
            "Total",
            name_width,
            arch,
            arch_width,
            totals.bss,
            totals.data,
            totals.text,
            section_width
        );
    }
}
