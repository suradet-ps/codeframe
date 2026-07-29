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
    """Draw a simplified CodeFrame icon with safe zone for adaptive icons."""
    pixels = bytearray(size * size * 4)

    def px(x, y, r, g, b, a=255):
        if 0 <= x < size and 0 <= y < size:
            i = (y * size + x) * 4
            pixels[i] = r
            pixels[i + 1] = g
            pixels[i + 2] = b
            pixels[i + 3] = a

    def fill_rounded_rect(x1, y1, w, h, radius, r, g, b):
        for y in range(y1, y1 + h):
            for x in range(x1, x1 + w):
                if x < 0 or x >= size or y < 0 or y >= size:
                    continue
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

    # Scale factor: all 512-space coords → actual size, then shrink to 80% safe zone
    s = (size * 0.80) / 512.0
    off = size * 0.10  # 10% margin on each side

    def v(val):
        """Map a 512-space coordinate to the safe zone."""
        return int(off + val * s)

    def vw(val):
        """Map a 512-space dimension (not coordinate)."""
        return int(val * s)

    # Background rounded rect (dark solid #09090b)
    fill_rounded_rect(0, 0, size, size, vw(116), 9, 9, 11)

    # White card
    fill_rounded_rect(v(88), v(64), vw(336), vw(384), vw(20), 255, 255, 255)

    # Code area (dark)
    fill_rounded_rect(v(116), v(92), vw(280), vw(240), vw(12), 15, 23, 42)

    # Traffic lights
    dot_r = max(1, vw(6))
    dot_y = v(118)
    for cx, (cr, cg, cb) in [
        (v(144), (239, 68, 68)),
        (v(164), (245, 158, 11)),
        (v(184), (16, 185, 129)),
    ]:
        for dy in range(-dot_r, dot_r + 1):
            for dx in range(-dot_r, dot_r + 1):
                if dx * dx + dy * dy <= dot_r * dot_r:
                    px(cx + dx, dot_y + dy, cr, cg, cb)

    # Code lines
    line_colors = [(56, 189, 248), (203, 213, 225), (192, 132, 252), (52, 211, 153)]
    line_y = [152, 182, 212, 242]
    line_w = [125, 200, 150, 105]
    line_h = vw(11)
    line_r = vw(6)
    for y, w, (lr, lg, lb) in zip(line_y, line_w, line_colors):
        fill_rounded_rect(v(144), v(y), vw(w), line_h, line_r, lr, lg, lb)

    # Bottom circle
    circle_r = vw(18)
    circle_cx = v(256)
    circle_cy = v(392)
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
