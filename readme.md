A tool for searching through pdf, docx files.

In the next updates will be added more files.

# Usage
```bash
cargo add
cargo build --release
```

and in `target/release` will be file `book_worm` that is working like a command tool.

## Working
book_worm <action> [param1 param2 ...]

actions:
	**scan**
		Scans all pdfs/docx files through your path and soon adds them to `database.db`

	**search**
		Search by phrase. Uses levenshtein and jaro distances. Returns sorted indexes of most relevant words and the name of page results.

## Troubleshoot
If something is broken, just delete database and scan the path again.