// Telegram WebApp initialization
const tg = window.Telegram.WebApp;
tg.ready();
tg.expand();

// API helper
const api = {
    async request(method, endpoint, body = null) {
        const options = {
            method,
            headers: {
                'Content-Type': 'application/json',
                'X-Telegram-Init-Data': tg.initData
            }
        };
        if (body) options.body = JSON.stringify(body);
        
        try {
            const res = await fetch(`/api${endpoint}`, options);
            const data = await res.json();
            if (!data.success) throw new Error(data.error || 'Unknown error');
            return data.data;
        } catch (e) {
            console.error('API Error:', e);
            tg.showAlert(e.message);
            throw e;
        }
    },
    get: (endpoint) => api.request('GET', endpoint),
    post: (endpoint, body) => api.request('POST', endpoint, body),
    put: (endpoint, body) => api.request('PUT', endpoint, body),
    delete: (endpoint) => api.request('DELETE', endpoint)
};

// Tab navigation
document.querySelectorAll('.nav-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        const tab = btn.dataset.tab;
        document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
        btn.classList.add('active');
        document.getElementById(tab).classList.add('active');
        loadTabData(tab);
    });
});

// Load tab data
async function loadTabData(tab) {
    switch (tab) {
        case 'status': await loadStatus(); break;
        case 'personas': await loadPersonas(); break;
        case 'chats': await loadChats(); break;
        case 'security': /* static content */ break;
        case 'config': await loadConfig(); break;
    }
}

// Status tab
async function loadStatus() {
    try {
        const status = await api.get('/status');
        
        // Update pause button
        updatePauseButton(status.paused);
        
        document.getElementById('ollama-status').textContent = status.ollama_online ? '🟢' : '🔴';
        document.getElementById('db-status').textContent = status.db_online ? '🟢' : '🔴';
        document.getElementById('active-persona').textContent = status.active_persona || 'Не выбрана';
        document.getElementById('queue-available').textContent = status.queue_available;
        document.getElementById('queue-max').textContent = status.queue_max;
        document.getElementById('total-requests').textContent = status.total_requests;
        document.getElementById('success-requests').textContent = status.successful_requests;
        document.getElementById('failed-requests').textContent = status.failed_requests;
        document.getElementById('avg-time').textContent = status.avg_response_time_ms;
        document.getElementById('model-name').textContent = status.model;
        document.getElementById('temperature').textContent = status.temperature;
        document.getElementById('max-tokens').textContent = status.max_tokens;
        
        const features = [];
        if (status.vision_enabled) features.push('👁️ Vision');
        if (status.voice_enabled) features.push('🎤 Voice');
        if (status.web_search_enabled) features.push('🌐 Web');
        document.getElementById('features-list').textContent = features.join(' • ') || 'Нет дополнительных функций';
    } catch (e) {
        console.error('Failed to load status:', e);
    }
}

// Pause functionality
function updatePauseButton(isPaused) {
    const btn = document.getElementById('pause-btn');
    const icon = document.getElementById('pause-icon');
    const text = document.getElementById('pause-text');
    
    if (isPaused) {
        btn.classList.add('paused');
        icon.textContent = '▶️';
        text.textContent = 'Возобновить';
    } else {
        btn.classList.remove('paused');
        icon.textContent = '⏸️';
        text.textContent = 'Пауза';
    }
}

async function togglePause() {
    try {
        const result = await api.post('/pause');
        updatePauseButton(result.paused);
        tg.showAlert(result.paused ? 'Бот приостановлен' : 'Бот возобновлён');
    } catch (e) {
        console.error('Failed to toggle pause:', e);
    }
}

// Personas tab
async function loadPersonas() {
    const list = document.getElementById('personas-list');
    list.innerHTML = '<div class="loading">Загрузка...</div>';
    
    try {
        const personas = await api.get('/personas');
        
        if (personas.length === 0) {
            list.innerHTML = '<div class="empty">Нет персон</div>';
            return;
        }
        
        list.innerHTML = personas.map(p => `
            <div class="list-item">
                <div class="list-item-header">
                    <span class="list-item-title">${escapeHtml(p.name)}</span>
                    <span class="badge ${p.is_active ? '' : 'badge-inactive'}">${p.is_active ? 'Активна' : 'ID: ' + p.id}</span>
                </div>
                <div class="list-item-subtitle">${escapeHtml(p.prompt.substring(0, 100))}${p.prompt.length > 100 ? '...' : ''}</div>
                <div class="list-item-actions">
                    ${!p.is_active ? `<button class="btn btn-small btn-primary" onclick="activatePersona(${p.id})">Активировать</button>` : ''}
                    <button class="btn btn-small btn-secondary" onclick="editPersona(${p.id}, '${escapeJs(p.name)}', '${escapeJs(p.prompt)}')">Изменить</button>
                    <button class="btn btn-small btn-danger" onclick="deletePersona(${p.id}, '${escapeJs(p.name)}')">Удалить</button>
                </div>
            </div>
        `).join('');
    } catch (e) {
        list.innerHTML = '<div class="empty">Ошибка загрузки</div>';
    }
}

function showCreatePersona() {
    showModal('Создать персону', `
        <div class="form-group">
            <label>Название</label>
            <input type="text" id="persona-name" placeholder="Например: Джарвис">
        </div>
        <div class="form-group">
            <label>Промпт (системное сообщение)</label>
            <textarea id="persona-prompt" placeholder="Опишите характер и поведение персоны..."></textarea>
        </div>
        <button class="btn btn-primary" onclick="createPersona()">Создать</button>
    `);
}

async function createPersona() {
    const name = document.getElementById('persona-name').value.trim();
    const prompt = document.getElementById('persona-prompt').value.trim();
    
    if (!name || !prompt) {
        tg.showAlert('Заполните все поля');
        return;
    }
    
    try {
        await api.post('/personas', { name, prompt });
        closeModal();
        await loadPersonas();
        tg.showAlert('Персона создана');
    } catch (e) {}
}

function editPersona(id, name, prompt) {
    showModal('Изменить персону', `
        <div class="form-group">
            <label>Название</label>
            <input type="text" id="persona-name" value="${escapeHtml(name)}">
        </div>
        <div class="form-group">
            <label>Промпт</label>
            <textarea id="persona-prompt">${escapeHtml(prompt)}</textarea>
        </div>
        <button class="btn btn-primary" onclick="updatePersona(${id})">Сохранить</button>
    `);
}

async function updatePersona(id) {
    const name = document.getElementById('persona-name').value.trim();
    const prompt = document.getElementById('persona-prompt').value.trim();
    
    if (!name || !prompt) {
        tg.showAlert('Заполните все поля');
        return;
    }
    
    try {
        await api.put(`/personas/${id}`, { name, prompt });
        closeModal();
        await loadPersonas();
    } catch (e) {}
}

async function activatePersona(id) {
    try {
        await api.post(`/personas/${id}/activate`);
        await loadPersonas();
        await loadStatus();
    } catch (e) {}
}

async function deletePersona(id, name) {
    tg.showConfirm(`Удалить персону "${name}"?`, async (confirmed) => {
        if (confirmed) {
            try {
                await api.post(`/personas/${id}/delete`);
                await loadPersonas();
            } catch (e) {}
        }
    });
}


// Chats tab
async function loadChats() {
    const list = document.getElementById('chats-list');
    list.innerHTML = '<div class="loading">Загрузка...</div>';
    
    try {
        const chats = await api.get('/chats');
        
        if (chats.length === 0) {
            list.innerHTML = '<div class="empty">Нет чатов</div>';
            return;
        }
        
        list.innerHTML = chats.map(c => `
            <div class="list-item">
                <div class="list-item-header">
                    <span class="list-item-title">Chat ${c.chat_id}</span>
                    <span class="badge ${c.auto_reply_enabled ? '' : 'badge-inactive'}">${c.auto_reply_enabled ? 'Активен' : 'Выключен'}</span>
                </div>
                <div class="list-item-subtitle">
                    ${c.reply_mode === 'all_messages' ? '💬 Все сообщения' : '👤 Только упоминания'} • 
                    RAG: ${c.rag_enabled ? '✅' : '❌'} • 
                    Cooldown: ${c.cooldown_seconds}с
                </div>
                <div class="list-item-actions">
                    <button class="btn btn-small btn-secondary" onclick="editChat(${c.chat_id})">Настройки</button>
                    <button class="btn btn-small btn-secondary" onclick="editTriggers(${c.chat_id})">Триггеры</button>
                </div>
            </div>
        `).join('');
    } catch (e) {
        list.innerHTML = '<div class="empty">Ошибка загрузки</div>';
    }
}

async function editChat(chatId) {
    try {
        const settings = await api.get(`/chats/${chatId}`);
        
        showModal(`Настройки чата ${chatId}`, `
            <div class="toggle-row">
                <span>Автоответы</span>
                <label class="toggle">
                    <input type="checkbox" id="auto-reply" ${settings.auto_reply_enabled ? 'checked' : ''}>
                    <span class="toggle-slider"></span>
                </label>
            </div>
            <div class="toggle-row">
                <span>RAG память</span>
                <label class="toggle">
                    <input type="checkbox" id="rag-enabled" ${settings.rag_enabled ? 'checked' : ''}>
                    <span class="toggle-slider"></span>
                </label>
            </div>
            <div class="form-group">
                <label>Режим ответов</label>
                <select id="reply-mode">
                    <option value="mention_only" ${settings.reply_mode === 'mention_only' ? 'selected' : ''}>Только упоминания</option>
                    <option value="all_messages" ${settings.reply_mode === 'all_messages' ? 'selected' : ''}>Все сообщения</option>
                </select>
            </div>
            <div class="form-group">
                <label>Cooldown (секунды)</label>
                <input type="number" id="cooldown" value="${settings.cooldown_seconds}" min="0" max="300">
            </div>
            <div class="form-group">
                <label>Глубина контекста</label>
                <input type="number" id="context-depth" value="${settings.context_depth}" min="1" max="50">
            </div>
            <button class="btn btn-primary" onclick="saveChatSettings(${chatId})">Сохранить</button>
        `);
    } catch (e) {}
}

async function saveChatSettings(chatId) {
    try {
        await api.put(`/chats/${chatId}`, {
            auto_reply_enabled: document.getElementById('auto-reply').checked,
            rag_enabled: document.getElementById('rag-enabled').checked,
            reply_mode: document.getElementById('reply-mode').value,
            cooldown_seconds: parseInt(document.getElementById('cooldown').value) || 5,
            context_depth: parseInt(document.getElementById('context-depth').value) || 10
        });
        closeModal();
        await loadChats();
    } catch (e) {}
}

async function editTriggers(chatId) {
    try {
        const triggers = await api.get(`/chats/${chatId}/triggers`);
        
        showModal(`Триггеры чата ${chatId}`, `
            <p style="color: var(--tg-theme-hint-color); margin-bottom: 16px;">
                Бот будет отвечать на сообщения, содержащие эти ключевые слова
            </p>
            <div class="form-group">
                <label>Ключевые слова (через запятую)</label>
                <textarea id="keywords" placeholder="помощь, вопрос, подскажи">${triggers.keywords.join(', ')}</textarea>
            </div>
            <button class="btn btn-primary" onclick="saveTriggers(${chatId})">Сохранить</button>
            <button class="btn btn-danger" onclick="clearTriggers(${chatId})">Очистить</button>
        `);
    } catch (e) {}
}

async function saveTriggers(chatId) {
    const input = document.getElementById('keywords').value;
    const keywords = input.split(',').map(k => k.trim().toLowerCase()).filter(k => k);
    
    try {
        await api.put(`/chats/${chatId}/triggers`, { keywords });
        closeModal();
        tg.showAlert('Триггеры сохранены');
    } catch (e) {}
}

async function clearTriggers(chatId) {
    try {
        await api.put(`/chats/${chatId}/triggers`, { keywords: [] });
        closeModal();
        tg.showAlert('Триггеры очищены');
    } catch (e) {}
}

// Settings tab
async function loadConfig() {
    await loadConfigForm();
    await loadStats();
}

async function loadConfigForm() {
    try {
        const cfg = await api.get('/config');
        
        document.getElementById('cfg-chat-model').value = cfg.ollama_chat_model;
        document.getElementById('cfg-embed-model').value = cfg.ollama_embedding_model;
        document.getElementById('cfg-vision-model').value = cfg.ollama_vision_model;
        document.getElementById('cfg-temperature').value = cfg.temperature;
        document.getElementById('cfg-max-tokens').value = cfg.max_tokens;
        document.getElementById('cfg-llm-timeout').value = cfg.llm_timeout_seconds;
        document.getElementById('cfg-max-concurrent').value = cfg.max_concurrent_llm_requests;
        document.getElementById('cfg-decay-rate').value = cfg.rag_decay_rate;
        document.getElementById('cfg-summary-threshold').value = cfg.summary_threshold;
        document.getElementById('cfg-vision-enabled').checked = cfg.vision_enabled;
        document.getElementById('cfg-voice-enabled').checked = cfg.voice_enabled;
        document.getElementById('cfg-web-search').checked = cfg.web_search_enabled;
        document.getElementById('cfg-random-reply').value = cfg.random_reply_probability;
    } catch (e) {
        console.error('Failed to load config:', e);
    }
}

async function saveConfig() {
    try {
        await api.put('/config', {
            ollama_chat_model: document.getElementById('cfg-chat-model').value,
            ollama_embedding_model: document.getElementById('cfg-embed-model').value,
            ollama_vision_model: document.getElementById('cfg-vision-model').value,
            temperature: parseFloat(document.getElementById('cfg-temperature').value),
            max_tokens: parseInt(document.getElementById('cfg-max-tokens').value),
            llm_timeout_seconds: parseInt(document.getElementById('cfg-llm-timeout').value),
            max_concurrent_llm_requests: parseInt(document.getElementById('cfg-max-concurrent').value),
            rag_decay_rate: parseFloat(document.getElementById('cfg-decay-rate').value),
            summary_threshold: parseInt(document.getElementById('cfg-summary-threshold').value),
            vision_enabled: document.getElementById('cfg-vision-enabled').checked,
            voice_enabled: document.getElementById('cfg-voice-enabled').checked,
            web_search_enabled: document.getElementById('cfg-web-search').checked,
            random_reply_probability: parseFloat(document.getElementById('cfg-random-reply').value),
        });
        tg.showAlert('Конфигурация сохранена');
    } catch (e) {}
}

async function loadStats() {
    const list = document.getElementById('stats-list');
    list.innerHTML = '<div class="loading">Загрузка...</div>';
    
    try {
        const stats = await api.get('/stats');
        
        if (stats.length === 0) {
            list.innerHTML = '<div class="empty">Нет статистики</div>';
            return;
        }
        
        list.innerHTML = stats.map(s => `
            <div class="list-item" style="flex-direction: row; justify-content: space-between; align-items: center;">
                <span>Chat ${s.chat_id}</span>
                <span class="badge">${s.message_count} сообщений</span>
            </div>
        `).join('');
    } catch (e) {
        list.innerHTML = '<div class="empty">Ошибка загрузки</div>';
    }
}

// Modal helpers
function showModal(title, content) {
    document.getElementById('modal-title').textContent = title;
    document.getElementById('modal-body').innerHTML = content;
    document.getElementById('modal').classList.remove('hidden');
}

function closeModal() {
    document.getElementById('modal').classList.add('hidden');
}

// Utility functions
function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

function escapeJs(str) {
    return str.replace(/\\/g, '\\\\').replace(/'/g, "\\'").replace(/\n/g, '\\n');
}

// Close modal on backdrop click
document.getElementById('modal').addEventListener('click', (e) => {
    if (e.target.id === 'modal') closeModal();
});

// Initial load
loadStatus();

// Security functions
let currentSecurityUserId = null;

async function checkUserSecurity() {
    const userId = document.getElementById('check-user-id').value;
    if (!userId) {
        tg.showAlert('Введите User ID');
        return;
    }

    try {
        const status = await api.get(`/security/users/${userId}`);
        currentSecurityUserId = parseInt(userId);
        
        const resultDiv = document.getElementById('security-result');
        const contentDiv = document.getElementById('security-result-content');
        
        const statusText = status.is_blocked ? '🔒 Заблокирован' : 
                          status.is_rate_limited ? '⏳ Rate Limited' : '✅ Активен';
        
        contentDiv.innerHTML = `
            <div>User ID: <strong>${status.user_id}</strong></div>
            <div>Статус: <strong>${statusText}</strong></div>
            <div>Текущие страйки: <strong>${status.strikes}/3</strong></div>
            <div>Всего нарушений: <strong>${status.total_violations}</strong></div>
        `;
        
        resultDiv.classList.remove('hidden');
        
        // Show/hide buttons based on status
        document.getElementById('block-user-btn').style.display = status.is_blocked ? 'none' : 'inline-block';
        document.getElementById('unblock-user-btn').style.display = status.is_blocked ? 'inline-block' : 'none';
    } catch (e) {
        document.getElementById('security-result').classList.add('hidden');
    }
}

async function blockUserFromCheck() {
    if (!currentSecurityUserId) return;
    
    tg.showConfirm(`Заблокировать пользователя ${currentSecurityUserId} на 30 минут?`, async (confirmed) => {
        if (confirmed) {
            try {
                await api.post(`/security/users/${currentSecurityUserId}/block`, { duration_minutes: 30 });
                tg.showAlert('Пользователь заблокирован');
                await checkUserSecurity();
            } catch (e) {}
        }
    });
}

async function unblockUserFromCheck() {
    if (!currentSecurityUserId) return;
    
    try {
        await api.post(`/security/users/${currentSecurityUserId}/unblock`, {});
        tg.showAlert('Пользователь разблокирован');
        await checkUserSecurity();
    } catch (e) {}
}
