function switchTab(tabId, el) {
    document.querySelectorAll('.auth-form').forEach(f => f.classList.remove('active'));
    document.querySelectorAll('.auth-tabs .btn').forEach(b => {
        b.classList.remove('btn-primary');
        b.classList.add('btn-outline');
    });
    document.getElementById(tabId).classList.add('active');
    el.classList.remove('btn-outline');
    el.classList.add('btn-primary');
}

async function submitForm(e, url) {
    e.preventDefault();
    const form = e.target;
    const msgEl = document.getElementById(form.id + '-msg');
    const formData = new FormData(form);
    const data = Object.fromEntries(formData.entries());

    msgEl.textContent = 'Loading...';
    msgEl.style.color = 'var(--color-info)';

    try {
        const res = await fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });

        if (res.ok) {
            const data = await res.json(); // Получаем JSON от бэкенда

            if (data.message) {
                // Это сработает при регистрации (покажет сообщение о письме)
                msgEl.textContent = data.message;
                msgEl.style.color = 'var(--color-success)';
                form.reset(); // Очищаем поля формы после успешной регистрации
            } else if (data.access_token) {
                // Это сработает при успешном входе
                localStorage.setItem('access_token', data.access_token);
                localStorage.setItem('user_id', data.user_id);
                localStorage.setItem('user_email', data.email);

                msgEl.textContent = 'Вход выполнен! Перенаправление...';
                msgEl.style.color = 'var(--color-success)';

                // Автоматический переход на dashboard через полсекунды
                setTimeout(() => {
                    window.location.href = 'dashboard.html';
                }, 500);
            } else {
                msgEl.textContent = 'Успешно!';
                msgEl.style.color = 'var(--color-success)';
            }
        } else {
            msgEl.textContent = 'Ошибка. Проверьте данные.';
            msgEl.style.color = 'var(--color-danger)';
        }
    } catch (err) {
        msgEl.textContent = 'Ошибка сети.';
        msgEl.style.color = 'var(--color-danger)';
    }
}