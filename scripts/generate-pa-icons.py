from PIL import Image, ImageDraw, ImageFont, ImageFilter
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
ICONS_DIR = BASE_DIR / "src-tauri" / "icons"
FONT_PATH = Path("C:/Windows/Fonts/arialbd.ttf")

def create_pa_icon(size: int) -> Image.Image:
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # 圆角矩形背景（蓝色）
    radius = size // 8
    bg_color = (37, 99, 235, 255)  # blue-600
    draw.rounded_rectangle([0, 0, size, size], radius=radius, fill=bg_color)

    # 文字 "PA"
    text = "PA"
    # 根据尺寸动态调整字号
    font_size = int(size * 0.55)
    font = ImageFont.truetype(str(FONT_PATH), font_size)

    bbox = draw.textbbox((0, 0), text, font=font)
    text_w = bbox[2] - bbox[0]
    text_h = bbox[3] - bbox[1]
    x = (size - text_w) / 2
    # 视觉上稍微往上偏移一点，让字母居中
    y = (size - text_h) / 2 - size * 0.03

    text_color = (255, 255, 255, 255)
    draw.text((x, y), text, font=font, fill=text_color)

    return img

def resize_sharp(img: Image.Image, size: int) -> Image.Image:
    resized = img.resize((size, size), Image.Resampling.LANCZOS)
    if size <= 64:
        resized = resized.filter(ImageFilter.SHARPEN)
    return resized

def main():
    base_size = 1024
    base = create_pa_icon(base_size)

    # 保存一份参考图
    ref_dir = BASE_DIR / "logo"
    ref_dir.mkdir(parents=True, exist_ok=True)
    base.save(ref_dir / "pa-icon.png")
    print("Saved logo/pa-icon.png")

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
        icon = resize_sharp(base, size)
        icon.save(ICONS_DIR / name)
        print(f"Generated {name} ({size}x{size})")

    # Windows ICO
    ico_sizes = [16, 20, 24, 32, 40, 48, 64, 96, 128, 180, 256]
    ico_images = [resize_sharp(base, s) for s in ico_sizes]
    ico_images[0].save(
        ICONS_DIR / "icon.ico",
        sizes=[(s, s) for s in ico_sizes],
        append_images=ico_images[1:],
    )
    print("Generated icon.ico")

    # macOS ICNS
    icns_sizes = [16, 32, 64, 128, 256, 512, 1024]
    icns_images = [resize_sharp(base, s) for s in icns_sizes]
    icns_images[0].save(
        ICONS_DIR / "icon.icns",
        sizes=[(s, s) for s in icns_sizes],
        append_images=icns_images[1:],
    )
    print("Generated icon.icns")

if __name__ == "__main__":
    main()
