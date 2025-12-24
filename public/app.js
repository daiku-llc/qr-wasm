const input = document.getElementById('qr-input');
const generateBtn = document.getElementById('generate-btn');
const loading = document.getElementById('loading');
const resultCard = document.getElementById('result-card');
const qrOutput = document.getElementById('qr-output');
const qrInfo = document.getElementById('qr-info');
const downloadBtn = document.getElementById('download-btn');

let currentQRData = null;

generateBtn.addEventListener('click', async () => {
    const text = input.value.trim();
    
    if (!text) {
        alert('Please enter some text or URL');
        return;
    }
    
    const format = document.querySelector('input[name="format"]:checked').value;
    
    // Show loading
    generateBtn.disabled = true;
    loading.classList.remove('hidden');
    resultCard.classList.add('hidden');
    
    try {
        const startTime = performance.now();
        
        const response = await fetch('/api/generate', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ data: text, format })
        });
        
        const endTime = performance.now();
        const duration = (endTime - startTime).toFixed(2);
        
        if (!response.ok) {
            throw new Error('Generation failed');
        }
        
        if (format === 'svg') {
            const svgText = await response.text();
            qrOutput.innerHTML = svgText;
            qrInfo.textContent = `Generated in ${duration}ms • Format: SVG • Scalable vector image`;
            currentQRData = { type: 'svg', data: svgText, text };
        } else {
            const json = await response.json();
            qrOutput.innerHTML = `<img src="${json.data_url}" alt="QR Code">`;
            qrInfo.textContent = `Generated in ${duration}ms • Format: PNG • Size: ${(json.size_bytes / 1024).toFixed(2)} KB`;
            currentQRData = { type: 'png', data: json.data_url, text };
        }
        
        resultCard.classList.remove('hidden');
    } catch (error) {
        alert('Error generating QR code: ' + error.message);
    } finally {
        generateBtn.disabled = false;
        loading.classList.add('hidden');
    }
});

downloadBtn.addEventListener('click', () => {
    if (!currentQRData) return;
    
    const link = document.createElement('a');
    
    if (currentQRData.type === 'svg') {
        const blob = new Blob([currentQRData.data], { type: 'image/svg+xml' });
        link.href = URL.createObjectURL(blob);
        link.download = 'qrcode.svg';
    } else {
        link.href = currentQRData.data;
        link.download = 'qrcode.png';
    }
    
    link.click();
});

// Allow Enter key to generate
input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && e.ctrlKey) {
        generateBtn.click();
    }
});

