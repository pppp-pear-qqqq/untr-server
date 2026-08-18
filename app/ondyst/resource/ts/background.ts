const canvas = document.getElementById('background') as HTMLCanvasElement;
const ctx = canvas.getContext('2d')!;
const imageSize = 180; // テクスチャ画像のサイズ
const tileSize = 120; // タイル1枚のサイズ
let textureImg: HTMLImageElement;

// 1. テクスチャ画像をロードする関数
async function loadTexture() {
    return new Promise((resolve) => {
        textureImg = new Image();
        // ここにベースとなるテクスチャ画像のURLを指定します
        // 今回はデモ用に、ざらざらしたコンクリートテクスチャを生成して使用します
        textureImg.src = 'image/back.png'; // (生成した画像のData URIをセット)
        textureImg.onload = resolve;
    });
}

// 2. RGBから明度(Luminance)を計算する関数 (0-255)
function getLuminance(r: number, g: number, b: number) {
    // 心理物理学的な明度の計算式
    return 0.299 * r + 0.587 * g + 0.114 * b;
}

// 3. HSLからRGBへ変換する関数 (0-255)
// h: [0, 360], s: [0, 100], l: [0, 100]
function hslToRgb(h: number, s: number, l: number) {
    const k = (n: number) => (n + h / 30) % 12;
    const a = s * Math.min(l, 1 - l);
    const f = (n: number) => l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
    return [255 * f(0), 255 * f(8), 255 * f(4)];
}

function drawBackground() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    const tileCols = Math.ceil(canvas.width / tileSize);
    const tileRows = Math.ceil(canvas.height / tileSize);

    // 1. まず、ランダムな色のタイルを敷き詰める
    ctx.globalCompositeOperation = 'source-over'; // デフォルトに戻す
    for (let y = 0; y < tileRows; y++) {
        for (let x = 0; x < tileCols; x++) {
            const hue = Math.floor(Math.random() * 360);
            // ※明度を50%にしておくと、テクスチャの明暗が綺麗に反映されます
            ctx.fillStyle = `hsl(${hue}, 50%, 50%)`;
            ctx.fillRect(x * tileSize, y * tileSize, tileSize, tileSize);
        }
    }

    // 2. 合成モードを「輝度（luminosity）」に変更する
    // これにより「下のレイヤーの色相・彩度」と「上のレイヤーの明度」が合成されます
    ctx.globalCompositeOperation = 'luminosity';

    // 3. テクスチャ画像をその上に敷き詰める
    const imageCols = Math.ceil(canvas.width / imageSize);
    const imageRows = Math.ceil(canvas.height / imageSize);
    for (let y = 0; y < imageRows; y++) {
        for (let x = 0; x < imageCols; x++) {
            ctx.drawImage(textureImg, x * imageSize, y * imageSize, imageSize, imageSize);
        }
    }
}

// 5. 初期化と実行
async function init() {
    await loadTexture();
    drawBackground();
    window.addEventListener('resize', drawBackground);
}

init();
