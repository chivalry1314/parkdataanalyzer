from PIL import Image, ImageFilter
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
SOURCE = BASE_DIR / "logo" / "logo.png"
ICONS_DIR = BASE_DIR / "src-tauri" / "icons"

def resize_sharp(img: Image.Image, size: int) -> Image.Image:
    """高质量缩放，小尺寸时额外锐化，避免发糊。"""
    resized = img.resize((size, size), Image.Resampling.LANCZOS)
    if size <= 64:
        # 小图标适当锐化边缘
        resized = resized.filter(ImageFilter.SHARPEN)
    return resized

def main():
    img = Image.open(SOURCE).convert("RGBA")

    sizes = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 256,
        "Square30x30Logo.png": 30,
        "Square44x44Logo.png": 44,
        "Square71x71Logo.png": 71,
        "Square89x89Logo.png": 89,
        "Square107x107Logo.png": 107,
        "Square142x142Logo.png": 142,
        "Square150x150Logo.png": 150,
        "Square284x284Logo.png": 284,
        "Square310x310Logo.png": 310,
        "StoreLogo.png": 50,
    }

    ICONS_DIR.mkdir(parents=True, exist_ok=True)

    for name, size in sizes.items():
        resized = resize_sharp(img, size)
        resized.save(ICONS_DIR / name)
        print(f"Generated {name} ({size}x{size})")

    # Windows ICO：包含更多常用尺寸，Windows 会按需取最接近的，不会强行放大缩小
    ico_sizes = [16, 20, 24, 32, 40, 48, 64, 96, 128, 180, 256]
    ico_images = [resize_sharp(img, s) for s in ico_sizes]
    ico_images[0].save(
        ICONS_DIR / "icon.ico",
        sizes=[(s, s) for s in ico_sizes],
        append_images=ico_images[1:],
    )
    print("Generated icon.ico")

    # macOS ICNS：生成一套标准尺寸后写入
    icns_sizes = [16, 32, 64, 128, 256, 512, 1024]
    icns_images = [resize_sharp(img, s) for s in icns_sizes]
    # Pillow 保存 icns 时会自动处理多尺寸
    icns_images[0].save(
        ICONS_DIR / "icon.icns",
        sizes=[(s, s) for s in icns_sizes],
        append_images=icns_images[1:],
    )
    print("Generated icon.icns")

if __name__ == "__main__":
    main()
