import os
from typing import Union
from sstn._norm import __normalize_text, __normalize_jsonl_file

def normalize_text(
    text : str,
) -> str:
    """
    Normalize the text by removing special characters and converting to lowercase.
    
    Args:
        text (str): The input text to be normalized.
    
    Returns:
        str: The normalized text.
    """
    return __normalize_text(text) # Call internal rust function

def normalize_jsonl_file(
    input_file : Union[str, os.PathLike],
    output_file : Union[str, os.PathLike],
    text_column : str = "text",
    workers : int = 1,
) -> None:
    """
    Normalize a JSONL file by applying text normalization to each line.
    
    Args:
        input_file (str): The path to the input JSONL file.
        output_file (str): The path to the output JSONL file.
    """
    __normalize_jsonl_file(input_file, output_file, text_column, workers) # Call internal rust function

def normalize_jsonl_files(
    paths : list[Union[str, os.PathLike]],
    output_dir : Union[str, os.PathLike],
    text_column : str = "text",
    workers : int = 1,
) -> None:
    """
    Normalize multiple JSONL files by applying text normalization to each line."
    
    Args:
        paths (list): A list of paths to the input JSONL files.
        output_dir (str): The directory where the output JSONL files will be saved.

    Information:
        The output files will be named the same as the input files, but in a different directory.
    """

    path_map = {
        path: os.path.join(output_dir, os.path.basename(path))
        for path in paths
    }

    # Make sure everything is in order
    os.makedirs(output_dir, exist_ok=True)

    for path in paths:
        if not os.path.exists(path):
            raise FileNotFoundError(f"File {path} does not exist.")
        if not os.path.isfile(path):
            raise IsADirectoryError(f"Path {path} is not a file.")
    
    for path in paths:
        __normalize_jsonl_file(path, path_map[path], text_column, workers)