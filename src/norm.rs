/// norm.rs
/// 
/// Provides functions for normalizing text.

use simd_json::OwnedValue;
use simd_json::prelude::*;
use std::io::Result;
use unidecode::unidecode;
use std::arch::x86_64::*;
use crate::set;
use rust_stemmers::{Algorithm, Stemmer};

fn remove_non_alphanumeric_scalar(text: &mut [u8]) -> () {
    // Base logic (1 byte at a time)
    for byte in text.iter_mut() {
        if !byte.is_ascii_alphanumeric() {
            *byte = b' ';
        }
    }
}

#[target_feature(enable = "sse4.1")]
unsafe fn remove_non_alphanumeric_sse2(text: &mut [u8]) -> () {
    // SSE2 logic (16 bytes at a time)
    
    let ptr : *mut u8 = text.as_mut_ptr();
    let len: usize = text.len();
    let mut i = 0;

    unsafe {
        let space = _mm_set1_epi8(b' ' as i8);

        while i + 16 <= len {
            // Load 16 bytes from the input
            let chunk = _mm_loadu_si128(ptr.add(i) as *const __m128i);

            let is_digit = {
                let digit_min = _mm_set1_epi8(b'0' as i8 - 1);
                let digit_max = _mm_set1_epi8(b'9' as i8 + 1);
                let ge_digit_min = _mm_cmpgt_epi8(chunk, digit_min);
                let le_digit_max = _mm_cmplt_epi8(chunk, digit_max);
                _mm_and_si128(ge_digit_min, le_digit_max)
            };

            let is_upper = {
                let upper_min = _mm_set1_epi8(b'A' as i8 - 1);
                let upper_max = _mm_set1_epi8(b'Z' as i8 + 1);
                let ge_upper_min = _mm_cmpgt_epi8(chunk, upper_min);
                let le_upper_max = _mm_cmplt_epi8(chunk, upper_max);
                _mm_and_si128(ge_upper_min, le_upper_max)
            };

            let is_lower = {
                let lower_min = _mm_set1_epi8(b'a' as i8 - 1);
                let lower_max = _mm_set1_epi8(b'z' as i8 + 1);
                let ge_lower_min = _mm_cmpgt_epi8(chunk, lower_min);
                let le_lower_max = _mm_cmplt_epi8(chunk, lower_max);
                _mm_and_si128(ge_lower_min, le_lower_max)
            };

            // Combine the masks
            let is_alphanumeric = _mm_or_si128(_mm_or_si128(is_digit, is_upper), is_lower);

            // Create a mask for non-alphanumeric characters
            let non_alphanumeric_mask = _mm_cmpeq_epi8(is_alphanumeric, _mm_setzero_si128());

            let result = _mm_blendv_epi8(chunk, space, non_alphanumeric_mask);
            // Store the result back to the output
            _mm_storeu_si128(ptr.add(i) as *mut __m128i, result);

            i += 16;
        }
    }
    
    
    // Process remaining bytes
    for j in i..len {
        if !text[j].is_ascii_alphanumeric() {
            text[j] = b' ';
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn remove_non_alphanumeric_avx2(text: &mut [u8]) -> () {
    // AVX2 logic (32 bytes at a time)
    let ptr : *mut u8 = text.as_mut_ptr();
    let len: usize = text.len();
    let mut i = 0;

    unsafe {
        let space = _mm256_set1_epi8(b' ' as i8);
        while i + 32 <= len {
            // Load 32 bytes from the input
            let chunk = _mm256_loadu_si256(ptr.add(i) as *const __m256i);

            let is_digit = {
                let digit_min = _mm256_set1_epi8(b'0' as i8 - 1); // We subtract 1 to include '0' since simd does not have less than or equal
                let digit_max = _mm256_set1_epi8(b'9' as i8 + 1); // Same here, we add 1 to include '9'
                let ge_digit_min = _mm256_cmpgt_epi8(chunk, digit_min); // chunk > '0' - 1
                let le_digit_max = _mm256_cmpgt_epi8(digit_max, chunk); // chunk < '9' + 1
                _mm256_and_si256(ge_digit_min, le_digit_max)
            };

            let is_upper = {
                let upper_min = _mm256_set1_epi8(b'A' as i8 - 1); // And so on...
                let upper_max = _mm256_set1_epi8(b'Z' as i8 + 1);
                let ge_upper_min = _mm256_cmpgt_epi8(chunk, upper_min); // chunk > 'A' - 1
                let le_upper_max = _mm256_cmpgt_epi8(upper_max, chunk); // chunk < 'Z' + 1
                _mm256_and_si256(ge_upper_min, le_upper_max)
            };

            let is_lower = {
                let lower_min = _mm256_set1_epi8(b'a' as i8 - 1);
                let lower_max = _mm256_set1_epi8(b'z' as i8 + 1);
                let ge_lower_min = _mm256_cmpgt_epi8(chunk, lower_min);
                let le_lower_max = _mm256_cmpgt_epi8(lower_max, chunk);
                _mm256_and_si256(ge_lower_min, le_lower_max)
            };

            // Combine the masks
            let is_alphanumeric = _mm256_or_si256(_mm256_or_si256(is_digit, is_upper), is_lower);

            // Create a mask for non-alphanumeric characters
            let non_alphanumeric_mask = _mm256_cmpeq_epi8(is_alphanumeric, _mm256_setzero_si256());

            let result = _mm256_blendv_epi8(chunk, space, non_alphanumeric_mask);
            // Store the result back to the output
            _mm256_storeu_si256(ptr.add(i) as *mut __m256i, result);

            i += 32;
        }
    }

    if i + 16 <= len {
        // If there are 16 bytes left, use SSE2 to process them
        unsafe { remove_non_alphanumeric_sse2(&mut text[i..]) };
        i += 16;
    }

    // Process remaining bytes
    for j in i..len {
        if !text[j].is_ascii_alphanumeric() {
            text[j] = b' ';
        }
    }
}

// TODO: avx512 (my machine doesn't support it so I can't test it)

/// Router for the remove_non_alphanumeric function
/// Uses the best available SIMD instruction set (excluding AVX512 for now)
fn remove_non_alphanumeric(text: &mut [u8]) -> () {
    // Call the SSE2 function
    if is_x86_feature_detected!("avx2") {
        unsafe { remove_non_alphanumeric_avx2(text) }
    } else
    if is_x86_feature_detected!("sse4.1") {
        // Fallback to the AVX2 logic
        unsafe { remove_non_alphanumeric_sse2(text) }
    } else {
        // Fallback to the base logic
        remove_non_alphanumeric_scalar(text)
    }
}


const SPACE : u8 = b' ';

/// Adds a word to the base string
/// If the word is empty, too short, or a stopword, it is skipped
/// 
/// # Arguments
/// * `base_string` - The base string to add the word to
/// * `word` - The word to add
/// * `stemmer` - The stemmer to use (rust-stemmers, porter2)
fn add_word(base_string : &mut String, word: &[u8], stemmer : &Stemmer) -> () {
    if word.is_empty() || word.len() < 2 {
        return;
    }

    // Ignore if all the characters are numeric
    if word.iter().all(|&c| c.is_ascii_digit()) {
        return;
    }

    let lowercase = word.to_ascii_lowercase(); // TODO: In-place this
    if set::STOPWORDS.contains(&lowercase) {
        return;
    }

    // Stem

    let as_str : &str = unsafe { std::str::from_utf8_unchecked(&lowercase) };
    let stemmed = stemmer.stem(as_str);

    base_string.push_str(stemmed.as_ref());
    base_string.push(SPACE as char);
}

/// Tokenizes a string into words
/// 
/// # Arguments
/// * `text` - The text to tokenize
/// 
/// # Warning
/// * It is not safe to use the buffer after this function is called
/// * This function expects the buffer to be a valid ASCII string.
/// If the string is not valid ASCII, we get undefined behavior
fn stem_text(text: &mut Vec<u8>) -> String {

    let mut tokenized_text = String::with_capacity(text.len());
    let stemmer = Stemmer::create(Algorithm::English);
    let mut start = 0;
    let mut in_word = false;
    for (i, &byte) in text.iter().enumerate() {
        if byte != SPACE && !in_word {
            // Start of a new word
            in_word = true;
            start = i;
        } else if byte == SPACE && in_word {
            // End of a word

            // Skip words that are too short
            if i - start < 2 {
                in_word = false;
                continue;
            }

            // Check if the word is camelCase
            for j in start..i-1 {
                if text[j].is_ascii_lowercase() && text[j+1].is_ascii_uppercase() {
                    // If the word is camelCase, split it
                    {
                        add_word(&mut tokenized_text, &text[start..j+1], &stemmer);
                    }
                    // Continue and let the next iteration handle the rest
                    start = j + 1;
                }
            }
            
            // Add the word to the tokenized text
            {
                add_word(&mut tokenized_text, &text[start..i], &stemmer);
            }
            in_word = false;

        }
    }

    tokenized_text
}

/// Public function to normalize text in a JSON object in-place.
/// 
/// # Arguments
/// * `val` - The JSON object to normalize
/// * `k` - The key to normalize
/// 
/// # Returns
/// * `Result<()>` - Ok if successful, Err if there was an error
pub fn normalize_owned_value(val : &mut OwnedValue, k : &str) -> Result<()>{

    let text = match val.get_str(k) {
        Some(text) => text,
        None => return Ok(()), // If the key doesn't exist, do nothing
    };

    let norm_text = _normalize_text(text)?;

    // SAFETY: We are replacing the text in the JSON object with a new string
    // and the JSON object is mutable
    match val.insert(k, norm_text) {
        Ok(_) => Ok(()),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("simd_json insert error: {:?}", e))),
    }

}

/// Normalizes a &str in-place
///
/// # Arguments
/// * `val` - The JSON object to normalize
/// 
/// # Returns
/// * `Result<String>` - The normalized string
pub fn _normalize_text(text : &str) -> Result<String> {
    
    // Turn text into ascii and then into a byte vector
    let ascii_text : String = unidecode(text);
    let mut ascii_vec : Vec<u8> = ascii_text.into_bytes();

    // We can use simd instructions to do SOME of the work

    // Turn non-alphanumeric characters into spaces
    remove_non_alphanumeric(&mut ascii_vec);

    let mut tokenized : String = stem_text(&mut ascii_vec);
    // Remove the last space
    tokenized.pop();


    // Turn multiple spaces into a single space
    // SAFETY: This is safe because the utf character set is a superset of ascii
    // And we are not adding non-ascii characters to the string
    // Handle the Result from insert; propagate any error
    Ok(tokenized)
}