"""Generate the Mark Linux liquid-glass macOS-style wallpaper."""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

TARGETS = [
    (Path("os/arch-base/profiles/desktop/airootfs/usr/share/marklinux/wallpapers/marklinux-glass.png"), 2560, 1600),
    (Path("os/arch-base/profiles/desktop/airootfs/usr/share/marklinux/wallpapers/marklinux-glass-16x9.png"), 1920, 1080),
]


def build(width: int, height: int) -> Image.Image:
    image = Image.new("RGBA", (width, height), (8, 11, 30, 255))
    blobs = [
        (width * 0.18, height * 0.34, width * 0.36, (64, 140, 255, 150)),
        (width * 0.78, height * 0.30, width * 0.30, (176, 82, 222, 150)),
        (width * 0.50, height * 0.78, width * 0.44, (32, 200, 210, 130)),
        (width * 0.86, height * 0.88, width * 0.30, (255, 120, 170, 120)),
        (width * 0.12, height * 0.95, width * 0.24, (38, 90, 200, 120)),
    ]
    for center_x, center_y, radius, color in blobs:
        mask = Image.new("L", (width, height), 0)
        draw = ImageDraw.Draw(mask)
        draw.ellipse(
            [center_x - radius, center_y - radius, center_x + radius, center_y + radius],
            fill=255,
        )
        mask = mask.filter(ImageFilter.GaussianBlur(radius=max(radius / 4.0, 2.0)))
        layer = Image.new("RGBA", (width, height), color)
        image = Image.composite(layer, image, mask)

    frost = Image.new("RGBA", (width, height), (255, 255, 255, 255))
    frost_mask = Image.new("L", (width, height), 0)
    frost_draw = ImageDraw.Draw(frost_mask)
    frost_draw.ellipse([width * 0.25, -height * 0.35, width * 0.75, height * 0.25], fill=255)
    frost_mask = frost_mask.filter(ImageFilter.GaussianBlur(radius=width * 0.30))
    image = Image.composite(frost, image, frost_mask)

    image = image.filter(ImageFilter.GaussianBlur(radius=min(width, height) * 0.01))
    image = image.convert("RGB")
    return image


def main() -> None:
    for path, width, height in TARGETS:
        path.parent.mkdir(parents=True, exist_ok=True)
        rendered = build(width, height)
        rendered.save(path, format="PNG", optimize=True)
        print(f"wrote {path} ({width}x{height})")


if __name__ == "__main__":
    main()
