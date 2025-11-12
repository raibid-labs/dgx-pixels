#!/usr/bin/env python3
"""
LoRA Validation Module

Generate validation samples to assess LoRA training quality and compare
against base model performance.
"""

import torch
from diffusers import StableDiffusionXLPipeline, DPMSolverMultistepScheduler
from peft import PeftModel
from PIL import Image
from pathlib import Path
from typing import List, Optional, Union, Dict
import logging
from datetime import datetime
import json
from tqdm import tqdm

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class LoRAValidator:
    """Validation tool for trained LoRA models"""

    def __init__(
        self,
        base_model: str = "stabilityai/stable-diffusion-xl-base-1.0",
        device: str = "cuda",
    ):
        self.base_model = base_model
        self.device = device
        self.pipeline = None

        logger.info(f"Initialized LoRAValidator with base model: {base_model}")

    def load_base_model(self):
        """Load base SDXL model"""
        if self.pipeline is not None:
            logger.info("Base model already loaded")
            return

        logger.info(f"Loading base model: {self.base_model}")

        # Check for local checkpoint
        checkpoint_path = Path("models/checkpoints/sd_xl_base_1.0.safetensors")
        if checkpoint_path.exists():
            logger.info(f"Loading from local checkpoint: {checkpoint_path}")
            self.pipeline = StableDiffusionXLPipeline.from_single_file(
                str(checkpoint_path),
                torch_dtype=torch.float16,
            )
        else:
            logger.info(f"Loading from HuggingFace: {self.base_model}")
            self.pipeline = StableDiffusionXLPipeline.from_pretrained(
                self.base_model,
                torch_dtype=torch.float16,
                variant="fp16",
            )

        # Optimize pipeline
        self.pipeline.scheduler = DPMSolverMultistepScheduler.from_config(
            self.pipeline.scheduler.config
        )
        self.pipeline.to(self.device)
        self.pipeline.enable_attention_slicing()

        logger.info("Base model loaded successfully")

    def load_lora(self, lora_path: Union[str, Path]):
        """Load LoRA weights into pipeline"""
        lora_path = Path(lora_path)

        if not lora_path.exists():
            raise ValueError(f"LoRA path does not exist: {lora_path}")

        logger.info(f"Loading LoRA from: {lora_path}")

        # Load LoRA into UNet
        self.pipeline.unet = PeftModel.from_pretrained(
            self.pipeline.unet,
            str(lora_path),
        )

        logger.info("LoRA loaded successfully")

    def unload_lora(self):
        """Remove LoRA weights and restore base model"""
        logger.info("Unloading LoRA weights")

        # Reload base model
        self.pipeline = None
        self.load_base_model()

    def generate(
        self,
        prompt: str,
        negative_prompt: Optional[str] = None,
        num_inference_steps: int = 25,
        guidance_scale: float = 8.0,
        seed: Optional[int] = None,
        width: int = 1024,
        height: int = 1024,
    ) -> Image.Image:
        """Generate image with current model configuration"""
        if self.pipeline is None:
            self.load_base_model()

        # Set seed for reproducibility
        if seed is not None:
            generator = torch.Generator(device=self.device).manual_seed(seed)
        else:
            generator = None

        # Default negative prompt for pixel art
        if negative_prompt is None:
            negative_prompt = (
                "blurry, smooth, gradient, 3d, realistic, photograph, "
                "low quality, distorted, dithering"
            )

        # Generate image
        with torch.no_grad():
            image = self.pipeline(
                prompt=prompt,
                negative_prompt=negative_prompt,
                num_inference_steps=num_inference_steps,
                guidance_scale=guidance_scale,
                generator=generator,
                width=width,
                height=height,
            ).images[0]

        return image

    def validate_lora(
        self,
        lora_path: Union[str, Path],
        validation_prompts: List[str],
        output_dir: Union[str, Path],
        compare_base: bool = True,
        num_inference_steps: int = 25,
        guidance_scale: float = 8.0,
        seeds: Optional[List[int]] = None,
    ) -> Dict[str, any]:
        """
        Generate validation samples for a trained LoRA

        Args:
            lora_path: Path to trained LoRA checkpoint
            validation_prompts: List of prompts to test
            output_dir: Directory to save validation images
            compare_base: Also generate with base model for comparison
            num_inference_steps: Number of inference steps
            guidance_scale: Guidance scale
            seeds: List of seeds (one per prompt)

        Returns:
            Validation results dictionary
        """
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        # Load base model
        self.load_base_model()

        # Use fixed seeds if not provided
        if seeds is None:
            seeds = [42 + i for i in range(len(validation_prompts))]

        # Generate with base model (if requested)
        if compare_base:
            logger.info("Generating with base model...")
            base_dir = output_dir / "base_model"
            base_dir.mkdir(exist_ok=True)

            for i, (prompt, seed) in enumerate(tqdm(zip(validation_prompts, seeds), total=len(validation_prompts))):
                image = self.generate(
                    prompt=prompt,
                    num_inference_steps=num_inference_steps,
                    guidance_scale=guidance_scale,
                    seed=seed,
                )

                # Save image
                image_path = base_dir / f"sample_{i:03d}.png"
                image.save(image_path)

                # Save prompt
                prompt_path = base_dir / f"sample_{i:03d}.txt"
                with open(prompt_path, 'w') as f:
                    f.write(f"Prompt: {prompt}\n")
                    f.write(f"Seed: {seed}\n")
                    f.write(f"Steps: {num_inference_steps}\n")
                    f.write(f"Guidance: {guidance_scale}\n")

        # Generate with LoRA
        logger.info("Generating with LoRA...")
        self.load_lora(lora_path)

        lora_dir = output_dir / "lora_model"
        lora_dir.mkdir(exist_ok=True)

        for i, (prompt, seed) in enumerate(tqdm(zip(validation_prompts, seeds), total=len(validation_prompts))):
            image = self.generate(
                prompt=prompt,
                num_inference_steps=num_inference_steps,
                guidance_scale=guidance_scale,
                seed=seed,
            )

            # Save image
            image_path = lora_dir / f"sample_{i:03d}.png"
            image.save(image_path)

            # Save prompt
            prompt_path = lora_dir / f"sample_{i:03d}.txt"
            with open(prompt_path, 'w') as f:
                f.write(f"Prompt: {prompt}\n")
                f.write(f"Seed: {seed}\n")
                f.write(f"Steps: {num_inference_steps}\n")
                f.write(f"Guidance: {guidance_scale}\n")

        # Create validation report
        report = {
            'lora_path': str(lora_path),
            'timestamp': datetime.now().isoformat(),
            'num_prompts': len(validation_prompts),
            'num_inference_steps': num_inference_steps,
            'guidance_scale': guidance_scale,
            'prompts': validation_prompts,
            'seeds': seeds,
            'compare_base': compare_base,
            'output_dir': str(output_dir),
        }

        # Save report
        report_path = output_dir / "validation_report.json"
        with open(report_path, 'w') as f:
            json.dump(report, f, indent=2)

        logger.info(f"Validation complete! Results saved to {output_dir}")

        return report

    def compare_models(
        self,
        lora_paths: List[Union[str, Path]],
        lora_names: List[str],
        validation_prompts: List[str],
        output_dir: Union[str, Path],
        num_inference_steps: int = 25,
        guidance_scale: float = 8.0,
        seeds: Optional[List[int]] = None,
    ):
        """
        Compare multiple LoRA models side-by-side

        Args:
            lora_paths: List of LoRA checkpoint paths
            lora_names: Names for each LoRA (for labeling)
            validation_prompts: Prompts to test
            output_dir: Output directory
            num_inference_steps: Inference steps
            guidance_scale: Guidance scale
            seeds: Seeds for each prompt
        """
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        if len(lora_paths) != len(lora_names):
            raise ValueError("Number of LoRA paths and names must match")

        # Use fixed seeds if not provided
        if seeds is None:
            seeds = [42 + i for i in range(len(validation_prompts))]

        # Load base model
        self.load_base_model()

        # Generate with each LoRA
        for lora_path, lora_name in zip(lora_paths, lora_names):
            logger.info(f"Generating with {lora_name}...")

            # Load LoRA
            self.unload_lora()
            self.load_lora(lora_path)

            # Create output directory
            model_dir = output_dir / lora_name
            model_dir.mkdir(exist_ok=True)

            # Generate images
            for i, (prompt, seed) in enumerate(tqdm(zip(validation_prompts, seeds), total=len(validation_prompts))):
                image = self.generate(
                    prompt=prompt,
                    num_inference_steps=num_inference_steps,
                    guidance_scale=guidance_scale,
                    seed=seed,
                )

                # Save image
                image_path = model_dir / f"sample_{i:03d}.png"
                image.save(image_path)

                # Save prompt
                prompt_path = model_dir / f"sample_{i:03d}.txt"
                with open(prompt_path, 'w') as f:
                    f.write(f"Model: {lora_name}\n")
                    f.write(f"Prompt: {prompt}\n")
                    f.write(f"Seed: {seed}\n")

        # Create comparison report
        report = {
            'timestamp': datetime.now().isoformat(),
            'models': {name: str(path) for name, path in zip(lora_names, lora_paths)},
            'prompts': validation_prompts,
            'seeds': seeds,
            'output_dir': str(output_dir),
        }

        report_path = output_dir / "comparison_report.json"
        with open(report_path, 'w') as f:
            json.dump(report, f, indent=2)

        logger.info(f"Model comparison complete! Results saved to {output_dir}")


def main():
    """CLI interface for LoRA validation"""
    import argparse

    parser = argparse.ArgumentParser(description="Validate trained LoRA models")
    parser.add_argument("--lora", type=str, required=True, help="Path to LoRA checkpoint")
    parser.add_argument("--output", type=str, required=True, help="Output directory")
    parser.add_argument("--prompts", type=str, nargs='+', help="Validation prompts")
    parser.add_argument("--prompts-file", type=str, help="File with prompts (one per line)")
    parser.add_argument("--compare-base", action="store_true", help="Also generate with base model")
    parser.add_argument("--steps", type=int, default=25, help="Inference steps")
    parser.add_argument("--guidance", type=float, default=8.0, help="Guidance scale")

    args = parser.parse_args()

    # Get prompts
    if args.prompts:
        prompts = args.prompts
    elif args.prompts_file:
        with open(args.prompts_file, 'r') as f:
            prompts = [line.strip() for line in f if line.strip()]
    else:
        # Default prompts
        prompts = [
            "pixel art knight character, standing pose, front view",
            "16bit potion item, red, game sprite",
            "pixel art mage casting spell, side view",
        ]

    # Create validator
    validator = LoRAValidator()

    # Run validation
    validator.validate_lora(
        lora_path=args.lora,
        validation_prompts=prompts,
        output_dir=args.output,
        compare_base=args.compare_base,
        num_inference_steps=args.steps,
        guidance_scale=args.guidance,
    )


if __name__ == "__main__":
    main()
