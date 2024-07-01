let currentColor = '#000000';
const pixelCanvas = document.getElementById('pixelCanvas');
const colorPicker = document.getElementById('colorPicker');

async function init() {
    const module = await import('./collaboration_canvas_client.js');
    await module.default();
    const canvas = new module.Canvas();

    await createPixelGrid(canvas);

    colorPicker.addEventListener('change', (event) => {
        currentColor = event.target.value;
    });
}

async function createPixelGrid(canvas) {
    for (let y = 0; y < 500; y++) {
        for (let x = 0; x < 50; x++) {
            const pixel = document.createElement('div');
            pixel.className = 'pixel';
            pixel.dataset.x = x;
            pixel.dataset.y = y;
            pixel.addEventListener('click', (e) => handlePixelClick(e, canvas));
            pixelCanvas.appendChild(pixel);
        }
        // Add a small delay every 10 rows to prevent browser freezing
        if (y % 10 === 0) {
            await new Promise(resolve => setTimeout(resolve, 0));
        }
    }
    console.log("Grid creation completed");
}

function handlePixelClick(event, canvas) {
    const x = parseInt(event.target.dataset.x);
    const y = parseInt(event.target.dataset.y);
    updatePixel(x, y, currentColor);
    canvas.update_pixel(x, y, currentColor);
}

function updatePixel(x, y, color) {
    const pixel = pixelCanvas.children[y * 50 + x];
    pixel.style.backgroundColor = color;
}

const ws = new WebSocket('ws://localhost:3030/ws');
ws.onmessage = (event) => {
    const update = JSON.parse(event.data);
    updatePixel(update.x, update.y, update.color);
};

init();