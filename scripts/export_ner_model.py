#!/usr/bin/env python3
"""
Export a Hugging Face NER model to ONNX format for use with the Rust engine.

Usage:
    python export_ner_model.py --model bert-base-NER --output model.onnx

Requirements:
    pip install transformers torch onnx optimum
"""

import argparse
import torch
from transformers import AutoTokenizer, AutoModelForTokenClassification
from pathlib import Path


def export_model(model_name: str, output_path: str, quantize: bool = True):
    """
    Export a Hugging Face NER model to ONNX format.

    Args:
        model_name: Name or path of the Hugging Face model
        output_path: Path to save the ONNX model
        quantize: Whether to quantize the model to int8
    """
    print(f"Loading model: {model_name}")

    # Load model and tokenizer
    tokenizer = AutoTokenizer.from_pretrained(model_name)
    model = AutoModelForTokenClassification.from_pretrained(model_name)

    # Set model to eval mode
    model.eval()

    # Create dummy input
    dummy_input = tokenizer(
        "This is a sample text",
        return_tensors="pt",
        padding="max_length",
        max_length=128,
        truncation=True
    )

    # Export to ONNX
    print(f"Exporting to ONNX: {output_path}")
    torch.onnx.export(
        model,
        (
            dummy_input["input_ids"],
            dummy_input["attention_mask"],
        ),
        output_path,
        input_names=["input_ids", "attention_mask"],
        output_names=["logits"],
        dynamic_axes={
            "input_ids": {0: "batch", 1: "sequence"},
            "attention_mask": {0: "batch", 1: "sequence"},
            "logits": {0: "batch", 1: "sequence"},
        },
        opset_version=14,
    )

    print(f"✓ Model exported successfully to {output_path}")

    # Quantize if requested
    if quantize:
        try:
            from onnxruntime.quantization import quantize_dynamic, QuantType

            quantized_path = output_path.replace(".onnx", "_quantized.onnx")
            print(f"Quantizing model to int8: {quantized_path}")

            quantize_dynamic(
                output_path,
                quantized_path,
                weight_type=QuantType.QInt8
            )

            print(f"✓ Model quantized successfully to {quantized_path}")

            # Show file sizes
            original_size = Path(output_path).stat().st_size / (1024 * 1024)
            quantized_size = Path(quantized_path).stat().st_size / (1024 * 1024)

            print(f"\nModel sizes:")
            print(f"  Original: {original_size:.2f} MB")
            print(f"  Quantized: {quantized_size:.2f} MB")
            print(f"  Reduction: {((1 - quantized_size/original_size) * 100):.1f}%")

        except ImportError:
            print("⚠ onnxruntime not installed, skipping quantization")
            print("  Install with: pip install onnxruntime")

    # Save tokenizer config
    tokenizer_path = output_path.replace(".onnx", "_tokenizer")
    tokenizer.save_pretrained(tokenizer_path)
    print(f"✓ Tokenizer saved to {tokenizer_path}")

    # Generate label mapping for Rust
    id2label = model.config.id2label
    label_mapping_path = output_path.replace(".onnx", "_labels.json")

    import json
    with open(label_mapping_path, 'w') as f:
        json.dump(id2label, f, indent=2)

    print(f"✓ Label mapping saved to {label_mapping_path}")
    print(f"\nLabels ({len(id2label)}):")
    for idx, label in sorted(id2label.items(), key=lambda x: int(x[0])):
        print(f"  {idx}: {label}")


def main():
    parser = argparse.ArgumentParser(
        description="Export Hugging Face NER model to ONNX"
    )
    parser.add_argument(
        "--model",
        type=str,
        default="dslim/bert-base-NER",
        help="Hugging Face model name or path"
    )
    parser.add_argument(
        "--output",
        type=str,
        default="models/ner_model.onnx",
        help="Output path for ONNX model"
    )
    parser.add_argument(
        "--no-quantize",
        action="store_true",
        help="Skip quantization step"
    )

    args = parser.parse_args()

    # Create output directory
    Path(args.output).parent.mkdir(parents=True, exist_ok=True)

    # Export model
    export_model(
        args.model,
        args.output,
        quantize=not args.no_quantize
    )

    print("\n✓ Export complete!")
    print(f"\nTo use in Rust:")
    print(f"  let recognizer = NerRecognizer::from_file(\"{args.output}\")?;")


if __name__ == "__main__":
    main()
