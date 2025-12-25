const input = document.getElementById('qr-input');
const generateBtn = document.getElementById('generate-btn');
const loading = document.getElementById('loading');
const resultCard = document.getElementById('result-card');
const qrOutput = document.getElementById('qr-output');
const qrInfo = document.getElementById('qr-info');
const downloadBtn = document.getElementById('download-btn');
const charCount = document.getElementById('char-count');
const byteCount = document.getElementById('byte-count');
const capacityStatus = document.getElementById('capacity-status');

let currentQRData = null;
let capacityCheckTimeout = null;

generateBtn.addEventListener('click', async () => {
    const text = input.value.trim();
    
    if (!text) {
        alert('Please enter some text or URL');
        return;
    }
    
    // Final capacity check before generating
    const byteCount = new TextEncoder().encode(text).length;
    if (byteCount > 2953) {
        alert(`Data too large: ${byteCount.toLocaleString()} bytes exceeds maximum capacity of 2,953 bytes. Please reduce by ${(byteCount - 2953).toLocaleString()} bytes.`);
        return;
    }
    
    // Show loading
    generateBtn.disabled = true;
    loading.classList.remove('hidden');
    resultCard.classList.add('hidden');
    
    try {
        const startTime = performance.now();
        
        const response = await fetch('/api/generate', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ data: text })
        });
        
        const endTime = performance.now();
        const duration = (endTime - startTime).toFixed(2);
        
        if (!response.ok) {
            // Try to get the actual error message from the response
            let errorMessage = 'Generation failed';
            try {
                const errorText = await response.text();
                if (errorText) {
                    // Try to parse as JSON first
                    try {
                        const errorJson = JSON.parse(errorText);
                        errorMessage = errorJson.error || errorJson.message || errorText;
                    } catch {
                        // If not JSON, use the text directly
                        errorMessage = errorText;
                    }
                } else {
                    errorMessage = `Generation failed: ${response.status} ${response.statusText}`;
                }
            } catch (e) {
                errorMessage = `Generation failed: ${response.status} ${response.statusText}`;
            }
            throw new Error(errorMessage);
        }
        
        const json = await response.json();
        qrOutput.innerHTML = `<img src="${json.data_url}" alt="QR Code">`;
        qrInfo.textContent = `Generated in ${duration}ms • Format: PNG • Size: ${(json.size_bytes / 1024).toFixed(2)} KB`;
        currentQRData = { type: 'png', data: json.data_url, text };
        
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
    link.href = currentQRData.data;
    link.download = 'qrcode.png';
    link.click();
});

// Real-time capacity checking
async function checkCapacity(text) {
    if (!text.trim()) {
        charCount.textContent = '0 characters';
        byteCount.textContent = '0 bytes';
        capacityStatus.textContent = '';
        capacityStatus.className = 'capacity-status';
        return;
    }
    
    try {
        const response = await fetch('/api/check-capacity', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ data: text })
        });
        
        if (response.ok) {
            const info = await response.json();
            
            charCount.textContent = `${info.char_count.toLocaleString()} characters`;
            byteCount.textContent = `${info.byte_count.toLocaleString()} bytes`;
            
            if (info.is_within_limit) {
                const remaining = info.max_capacity_bytes - info.byte_count;
                capacityStatus.textContent = `✓ ${remaining.toLocaleString()} bytes remaining (${info.percentage_used}% used)`;
                capacityStatus.className = 'capacity-status within-limit';
                generateBtn.disabled = false;
            } else {
                capacityStatus.textContent = `✗ ${info.bytes_over.toLocaleString()} bytes over limit`;
                capacityStatus.className = 'capacity-status over-limit';
                generateBtn.disabled = true;
            }
        }
    } catch (error) {
        // Fallback to client-side calculation if API fails
        const textBytes = new TextEncoder().encode(text).length;
        const textChars = text.length;
        const maxBytes = 2953;
        
        charCount.textContent = `${textChars.toLocaleString()} characters`;
        byteCount.textContent = `${textBytes.toLocaleString()} bytes`;
        
        if (textBytes <= maxBytes) {
            const remaining = maxBytes - textBytes;
            capacityStatus.textContent = `✓ ${remaining.toLocaleString()} bytes remaining`;
            capacityStatus.className = 'capacity-status within-limit';
            generateBtn.disabled = false;
        } else {
            const over = textBytes - maxBytes;
            capacityStatus.textContent = `✗ ${over.toLocaleString()} bytes over limit`;
            capacityStatus.className = 'capacity-status over-limit';
            generateBtn.disabled = true;
        }
    }
}

// Update capacity info as user types (debounced)
input.addEventListener('input', (e) => {
    const text = e.target.value;
    
    // Clear previous timeout
    if (capacityCheckTimeout) {
        clearTimeout(capacityCheckTimeout);
    }
    
    // Debounce API calls (wait 300ms after user stops typing)
    capacityCheckTimeout = setTimeout(() => {
        checkCapacity(text);
    }, 300);
    
    // Update character count immediately (no API call needed)
    const chars = text.length;
    charCount.textContent = `${chars.toLocaleString()} characters`;
});

// Check capacity on page load if there's existing text
if (input.value) {
    checkCapacity(input.value);
}

// Allow Enter key to generate
input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && e.ctrlKey) {
        generateBtn.click();
    }
});

