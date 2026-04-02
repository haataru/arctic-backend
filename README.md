# arctic-backend

<div align="center">
  <img src="https://i.postimg.cc/rpGyj8Qm/Snimok-ekrana-2026-04-02-154406.png" width="800" />
  <p><em>Главная страница со списком треков</em></p>
</div>

<div align="center">
  <img src="https://i.postimg.cc/J7YrPYcQ/Snimok-ekrana-2026-04-02-154431.png" width="800" />
  <p><em>Форма загрузки трека</em></p>
</div>

<div align="center">
  <img src="https://i.postimg.cc/8558vD3z/photo-2026-04-02-01-18-45.jpg" />
  <p><em>API</em></p>
</div>

## О проекте

Бэкенд на **Rust (Actix-web)** + фронтенд на **Next.js**.
- Нет нормализации звука - как задумал автор
- Перемотка через range requests
- Загрузка треков и обложек
- SoundCloud-like интерфейс

## Быстрый старт

### Backend (Rust)
```bash
cd backend
cargo run
```
Сервер: `http://localhost:8080`

### Frontend (Next.js)
```bash
cd frontend
npm install
npm run dev
```
Сервер: `http://localhost:3000`

---

## Репозитории

- [Backend (Rust)](https://github.com/haataru/arctic-backend)
- [Frontend (Next.js)](https://github.com/haataru/arctic-frontend)
