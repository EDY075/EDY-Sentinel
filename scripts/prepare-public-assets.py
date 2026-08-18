#!/usr/bin/env python3
"""Build public EDY Sentinel repository images from real QA screenshots.

The script never synthesizes telemetry. It crops or visibly redacts endpoint-specific
identity, process, software, and network details before creating repository assets.
"""

from __future__ import annotations

import argparse
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont


BACKGROUND = "#08111d"
SURFACE = "#101c2b"
BORDER = "#29415d"
BLUE = "#4c8dff"
TEXT = "#edf5ff"
MUTED = "#99acc4"
GREEN = "#36cfaa"


def font(size: int, *, bold: bool = False) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    candidates = [
        Path("C:/Windows/Fonts/seguisb.ttf" if bold else "C:/Windows/Fonts/segoeui.ttf"),
        Path("C:/Windows/Fonts/segoeuib.ttf" if bold else "C:/Windows/Fonts/segoeui.ttf"),
    ]
    for candidate in candidates:
        if candidate.exists():
            return ImageFont.truetype(str(candidate), size)
    return ImageFont.load_default()


def save_png(image: Image.Image, target: Path) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    image.convert("RGB").save(target, format="PNG", optimize=True, compress_level=9)


def redact(image: Image.Image, box: tuple[int, int, int, int], label: str) -> None:
    draw = ImageDraw.Draw(image)
    draw.rounded_rectangle(box, radius=10, fill="#0b1522", outline=BORDER, width=2)
    label_font = font(16, bold=True)
    bounds = draw.textbbox((0, 0), label, font=label_font)
    width = bounds[2] - bounds[0]
    height = bounds[3] - bounds[1]
    x = box[0] + (box[2] - box[0] - width) // 2
    y = box[1] + (box[3] - box[1] - height) // 2
    draw.text((x, y), label, font=label_font, fill=MUTED)


def load(source: Path) -> Image.Image:
    if not source.exists():
        raise FileNotFoundError(source)
    return Image.open(source).convert("RGB")


def fit_panel(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    target_width, target_height = size
    ratio = min(target_width / image.width, target_height / image.height)
    resized = image.resize(
        (max(1, round(image.width * ratio)), max(1, round(image.height * ratio))),
        Image.Resampling.LANCZOS,
    )
    canvas = Image.new("RGB", size, BACKGROUND)
    x = (target_width - resized.width) // 2
    y = (target_height - resized.height) // 2
    canvas.paste(resized, (x, y))
    return canvas


def build_screenshots(sprint5c: Path, sprint5a: Path, output: Path) -> dict[str, Path]:
    screenshots = output / "screenshots"
    screenshots.mkdir(parents=True, exist_ok=True)

    overview = load(sprint5c / "08-overview-english-sentinel-blue.png").crop((0, 0, 1440, 620))
    redact(overview, (264, 246, 636, 328), "Endpoint identity redacted")
    overview_target = screenshots / "overview-sentinel-blue.png"
    save_png(overview, overview_target)

    spectrum = load(sprint5c / "07-overview-spectrum.png").crop((0, 0, 1440, 620))
    redact(spectrum, (264, 246, 636, 328), "Endpoint identity redacted")
    spectrum_target = screenshots / "overview-spectrum.png"
    save_png(spectrum, spectrum_target)

    inventory = load(sprint5c / "02-inventory-sentinel-blue.png").crop((0, 0, 1440, 620))
    redact(inventory, (245, 314, 1397, 610), "Endpoint software rows redacted")
    inventory_target = screenshots / "software-inventory.png"
    save_png(inventory, inventory_target)

    network = load(sprint5a / "network-en-cyber-green-1440x900.png").crop((0, 31, 1440, 551))
    redact(network, (268, 340, 1396, 510), "Endpoint connection details redacted")
    network_target = screenshots / "network-telemetry.png"
    save_png(network, network_target)

    score = load(sprint5c / "03-security-score-sentinel-blue.png").crop((780, 0, 1440, 900))
    score_target = screenshots / "security-score-v2.png"
    save_png(score, score_target)

    inspector = load(sprint5c / "04-endpoint-inspector-sentinel-blue.png").crop((780, 0, 1440, 900))
    inspector_target = screenshots / "endpoint-inspector.png"
    save_png(inspector, inspector_target)

    return {
        "Overview": overview_target,
        "Network telemetry": network_target,
        "Software Inventory": inventory_target,
        "Security Score v2": score_target,
        "Endpoint Inspector": inspector_target,
        "Spectrum theme": spectrum_target,
    }


def build_banner(icon_path: Path, overview_path: Path, target: Path) -> None:
    width, height = 1600, 600
    banner = Image.new("RGB", (width, height), BACKGROUND)
    pixels = banner.load()
    for y in range(height):
        for x in range(width):
            mix = (x / width) * 0.7 + (y / height) * 0.3
            pixels[x, y] = (
                round(7 + 9 * mix),
                round(16 + 19 * mix),
                round(29 + 30 * mix),
            )

    draw = ImageDraw.Draw(banner)
    for x in range(0, width, 80):
        draw.line((x, 0, x - 240, height), fill="#10243a", width=1)
    draw.rectangle((0, 0, 12, height), fill=BLUE)
    draw.rectangle((12, 0, 18, height), fill=GREEN)

    icon = load(icon_path).resize((92, 92), Image.Resampling.LANCZOS)
    banner.paste(icon, (92, 72))
    draw.text((205, 78), "EDY SENTINEL", font=font(43, bold=True), fill=TEXT)
    draw.text((205, 132), "WINDOWS ENDPOINT INTELLIGENCE", font=font(18, bold=True), fill=BLUE)
    draw.text((92, 205), "See the endpoint. Explain the posture.", font=font(30, bold=True), fill=TEXT)
    draw.text(
        (92, 254),
        "Local-first Windows telemetry, detections, vulnerability intelligence,\nand an auditable Security Score.",
        font=font(19),
        fill=MUTED,
        spacing=8,
    )

    pills = [("REAL TELEMETRY", BLUE), ("LOCAL-FIRST", GREEN), ("EXPLAINABLE SCORE", BLUE)]
    x = 92
    for label, color in pills:
        label_font = font(15, bold=True)
        box = draw.textbbox((0, 0), label, font=label_font)
        pill_width = box[2] - box[0] + 34
        draw.rounded_rectangle((x, 350, x + pill_width, 389), radius=10, fill="#101f31", outline=color, width=1)
        draw.text((x + 17, 359), label, font=label_font, fill=color)
        x += pill_width + 12

    draw.text((92, 454), "Windows 10/11  •  English + Português (Brasil)  •  4 themes", font=font(17), fill=TEXT)
    draw.text((92, 500), "Open source desktop security observability", font=font(16), fill=MUTED)

    overview = load(overview_path)
    screenshot = fit_panel(overview, (850, 430)).resize((850, 430), Image.Resampling.LANCZOS)
    shadow = Image.new("RGBA", (910, 490), (0, 0, 0, 0))
    shadow_draw = ImageDraw.Draw(shadow)
    shadow_draw.rounded_rectangle((25, 25, 885, 465), radius=24, fill=(0, 0, 0, 175))
    shadow = shadow.filter(ImageFilter.GaussianBlur(18))
    banner.paste(shadow, (665, 55), shadow)
    frame = Image.new("RGB", (874, 454), SURFACE)
    frame.paste(screenshot, (12, 12))
    mask = Image.new("L", frame.size, 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, frame.width, frame.height), radius=18, fill=255)
    banner.paste(frame, (680, 65), mask)
    draw.rounded_rectangle((680, 65, 1554, 519), radius=18, outline="#416b9b", width=2)
    save_png(banner, target)


def gif_frame(image: Image.Image, label: str) -> Image.Image:
    canvas = Image.new("RGB", (960, 540), BACKGROUND)
    draw = ImageDraw.Draw(canvas)
    draw.text((28, 18), label, font=font(22, bold=True), fill=TEXT)
    draw.text((930, 23), "REAL APP CAPTURE", anchor="ra", font=font(12, bold=True), fill=GREEN)
    panel = fit_panel(image, (920, 470))
    canvas.paste(panel, (20, 58))
    draw.rounded_rectangle((19, 57, 941, 529), radius=12, outline=BORDER, width=2)
    return canvas


def build_gif(screenshots: dict[str, Path], target: Path) -> None:
    sequence = ["Overview", "Network telemetry", "Software Inventory", "Security Score v2"]
    base_frames = [gif_frame(load(screenshots[label]), label) for label in sequence]
    frames: list[Image.Image] = []
    durations: list[int] = []
    for index, current in enumerate(base_frames):
        frames.append(current)
        durations.append(1750)
        if index < len(base_frames) - 1:
            frames.append(Image.blend(current, base_frames[index + 1], 0.5))
            durations.append(250)

    palette_frames = [frame.convert("P", palette=Image.Palette.ADAPTIVE, colors=128) for frame in frames]
    target.parent.mkdir(parents=True, exist_ok=True)
    palette_frames[0].save(
        target,
        save_all=True,
        append_images=palette_frames[1:],
        duration=durations,
        loop=0,
        optimize=True,
        disposal=2,
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--sprint5c", type=Path, required=True)
    parser.add_argument("--sprint5a", type=Path, required=True)
    parser.add_argument("--icon", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=Path("docs/assets"))
    args = parser.parse_args()

    screenshots = build_screenshots(args.sprint5c, args.sprint5a, args.output)
    build_banner(args.icon, screenshots["Overview"], args.output / "edy-sentinel-banner.png")
    build_gif(screenshots, args.output / "edy-sentinel-demo.gif")

    for label, path in screenshots.items():
        print(f"{label}: {path} ({path.stat().st_size} bytes)")
    for path in (args.output / "edy-sentinel-banner.png", args.output / "edy-sentinel-demo.gif"):
        print(f"{path}: {path.stat().st_size} bytes")


if __name__ == "__main__":
    main()
