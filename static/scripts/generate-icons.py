"""Generate PWA icons as simple branded PNGs using only stdlib."""
import struct
import zlib
import os

def create_png(width, height, pixels):
    """Create a valid PNG file from raw RGBA pixel data."""
    def chunk(chunk_type, data):
        c = chunk_type + data
        crc = struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)
        return struct.pack(">I", len(data)) + c + crc

    raw = b""
    for y in range(height):
        raw += b"\x00"  # filter none
        raw += pixels[y * width * 4 : (y + 1) * width * 4]

    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    idat = chunk(b"IDAT", zlib.compress(raw))
    iend = chunk(b"IEND", b"")
    return sig + ihdr + idat + iend


def draw_icon(size):
    """Draw a simplified CodeFrame icon."""
    pixels = bytearray(size * size * 4)

    def px(x, y, r, g, b, a=255):
        if 0 <= x < size and 0 <= y < size:
            i = (y * size + x) * 4
            pixels[i] = r
            pixels[i + 1] = g
            pixels[i + 2] = b
            pixels[i + 3] = a

    def fill_rect(x1, y1, w, h, r, g, b):
        for y in range(y1, min(y1 + h, size)):
            for x in range(x1, min(x1 + w, size)):
                px(x, y, r, g, b)

    def fill_rounded_rect(x1, y1, w, h, radius, r, g, b):
        for y in range(y1, min(y1 + h, size)):
            for x in range(x1, min(x1 + w, size)):
                # Check corners
                in_rect = True
                if x < x1 + radius and y < y1 + radius:
                    dx = x1 + radius - x
                    dy = y1 + radius - y
                    if dx * dx + dy * dy > radius * radius:
                        in_rect = False
                elif x >= x1 + w - radius and y < y1 + radius:
                    dx = x - (x1 + w - radius - 1)
                    dy = y1 + radius - y
                    if dx * dx + dy * dy > radius * radius:
                        in_rect = False
                elif x < x1 + radius and y >= y1 + h - radius:
                    dx = x1 + radius - x
                    dy = y - (y1 + h - radius - 1)
                    if dx * dx + dy * dy > radius * radius:
                        in_rect = False
                elif x >= x1 + w - radius and y >= y1 + h - radius:
                    dx = x - (x1 + w - radius - 1)
                    dy = y - (y1 + h - radius - 1)
                    if dx * dx + dy * dy > radius * radius:
                        in_rect = False
                if in_rect:
                    px(x, y, r, g, b)

    s = size / 512.0

    # Background rounded rect (dark gradient approximated as solid #09090b)
    fill_rounded_rect(0, 0, size, size, int(116 * s), 9, 9, 11)

    # White card
    fill_rounded_rect(
        int(88 * s), int(64 * s), int(336 * s), int(384 * s), int(20 * s), 255, 255, 255
    )

    # Code area (dark)
    fill_rounded_rect(
        int(116 * s), int(92 * s), int(280 * s), int(240 * s), int(12 * s), 15, 23, 42
    )

    # Traffic lights
    dot_r = int(6 * s)
    dot_y = int(118 * s)
    for cx, (cr, cg, cb) in [
        (int(144 * s), (239, 68, 68)),
        (int(164 * s), (245, 158, 11)),
        (int(184 * s), (16, 185, 129)),
    ]:
        for dy in range(-dot_r, dot_r + 1):
            for dx in range(-dot_r, dot_r + 1):
                if dx * dx + dy * dy <= dot_r * dot_r:
                    px(cx + dx, dot_y + dy, cr, cg, cb)

    # Code lines
    line_colors = [(56, 189, 248), (203, 213, 225), (192, 132, 252), (52, 211, 153)]
    line_y = [152, 182, 212, 242]
    line_w = [125, 200, 150, 105]
    line_h = int(11 * s)
    line_r = int(5.5 * s)
    for i, (y, w, (lr, lg, lb)) in enumerate(zip(line_y, line_w, line_colors)):
        fill_rounded_rect(
            int(144 * s), int(y * s), int(w * s), line_h, line_r, lr, lg, lb
        )

    # Bottom circle
    circle_r = int(18 * s)
    circle_cx = int(256 * s)
    circle_cy = int(392 * s)
    for dy in range(-circle_r, circle_r + 1):
        for dx in range(-circle_r, circle_r + 1):
            if dx * dx + dy * dy <= circle_r * circle_r:
                px(circle_cx + dx, circle_cy + dy, 113, 113, 122)

    return bytes(pixels)


for size in [192, 512]:
    pixels = draw_icon(size)
    png_data = create_png(size, size, pixels)
    out_path = os.path.join(os.path.dirname(__file__), "..", "pwa", f"icon-{size}.png")
    with open(out_path, "wb") as f:
        f.write(png_data)
    print(f"Generated icon-{size}.png ({len(png_data)} bytes)")
