## Super Simple Text Normalizer
Super Simple Text Normalizer — a fast, lightweight text normalization tool designed for preprocessing text in NLP pipelines. sstn applies a minimal, efficient set of transformations to bring raw text into a consistent format, leveraging SIMD acceleration where possible.

This project was designed for performance in very large datasets, so there is a bias towards lightweight transformations. If there is something you need, open an issue!

## Features

✅ Unicode to ASCII conversion

✅ Removal of non-alphanumeric characters

✅ Removal of lone numbers (e.g. 2025, 123)

✅ Stopword removal

✅ CamelCase splitting → camelCase → camel case

✅ Lowercasing

✅ Porter2 stemming

⚡ SIMD acceleration with SSE4.1 and AVX2 for some operations (fallback to scalar when unavailable)

## Installation

(This part is still WIP)

```
git clone https://github.com/roloza7/sstn
cd sstn
pip install maturin
maturin develop -r
```

## Usage

One can use it to normalize strings:

```python
import sstn

sample = "Hello, I am a sample string with not a lot of punctuation."
sstn.normalize_text(sample) # "hello sampl string lot punctuat"

```

 But the main time saver is when normalizing whole files:
 
```python
import sstn

sstn.normalize_jsonl_file(
    input_file="sample.jsonl.gz",
    output_file="sample-normalized.jsonl.gz"
    text_column="text",
    workers=4
)
```


## Feature Requests & Contributions
Have an idea for a feature you'd like to see?
Open an issue! I'm actively maintaining the project and happy to consider useful additions.

## Planned Features
- [ ] Configurable pipeline: enable/disable individual normalization steps
- [ ] SIMD support for ARM (NEON)
- [ ] Lemmatization and POS tagging (if fast implementations can be found
- [ ] PyPI wheels

## Benchmarks

Not a lot here yet. Normalizing [Dolma](https://huggingface.co/datasets/allenai/dolma/)'s `cc_en_middle-0577.json.gz` (1.7M documents) takes ~96.68 seconds with 16 workers and AVX2 capability, with a time per document of 56 microseconds.
