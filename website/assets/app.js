document.addEventListener('DOMContentLoaded', () => {
  // Clipboard copy handler
  const copyBtns = document.querySelectorAll('.copy-btn');
  copyBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      const targetId = btn.getAttribute('data-clipboard-target');
      let text = '';
      if (targetId) {
        const el = document.getElementById(targetId);
        text = el ? el.innerText : '';
      } else {
        text = btn.getAttribute('data-clipboard-text') || '';
      }

      if (text) {
        navigator.clipboard.writeText(text).then(() => {
          const original = btn.innerText;
          btn.innerText = 'Copied!';
          btn.style.color = '#34d399';
          setTimeout(() => {
            btn.innerText = original;
            btn.style.color = '';
          }, 2000);
        });
      }
    });
  });

  // Tab switching logic for preview showcases
  const tabButtons = document.querySelectorAll('[data-tab-group]');
  tabButtons.forEach(btn => {
    btn.addEventListener('click', () => {
      const group = btn.getAttribute('data-tab-group');
      const target = btn.getAttribute('data-tab-target');

      document.querySelectorAll(`[data-tab-group="${group}"]`).forEach(b => b.classList.remove('active'));
      document.querySelectorAll(`[data-tab-content="${group}"]`).forEach(c => c.style.display = 'none');

      btn.classList.add('active');
      const activeContent = document.getElementById(target);
      if (activeContent) {
        activeContent.style.display = 'block';
      }
    });
  });
});
