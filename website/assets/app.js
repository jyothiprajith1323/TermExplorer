'use strict';

(() => {
  const ready = () => {
    const menuButton = document.querySelector('.menu-toggle');
    const nav = document.getElementById('primary-nav');

    if (menuButton && nav) {
      const closeMenu = () => {
        nav.classList.remove('is-open');
        menuButton.setAttribute('aria-expanded', 'false');
        menuButton.setAttribute('aria-label', 'Open navigation');
      };
      menuButton.addEventListener('click', () => {
        const open = menuButton.getAttribute('aria-expanded') !== 'true';
        nav.classList.toggle('is-open', open);
        menuButton.setAttribute('aria-expanded', String(open));
        menuButton.setAttribute('aria-label', open ? 'Close navigation' : 'Open navigation');
      });
      nav.addEventListener('click', event => {
        if (event.target.closest('a')) closeMenu();
      });
      document.addEventListener('keydown', event => {
        if (event.key === 'Escape' && nav.classList.contains('is-open')) {
          closeMenu();
          menuButton.focus();
        }
      });
      document.addEventListener('click', event => {
        if (!nav.contains(event.target) && !menuButton.contains(event.target)) closeMenu();
      });
      window.addEventListener('resize', () => {
        if (window.matchMedia('(min-width: 861px)').matches) closeMenu();
      }, { passive: true });
    }

    document.querySelectorAll('[data-year]').forEach(el => {
      el.textContent = String(new Date().getFullYear());
    });

    document.querySelectorAll('pre').forEach(el => {
      if (!el.hasAttribute('tabindex')) {
        el.setAttribute('tabindex', '0');
      }
    });

    const copyText = async text => {
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(text);
        return;
      }
      const field = document.createElement('textarea');
      field.value = text;
      field.setAttribute('readonly', '');
      field.style.position = 'fixed';
      field.style.opacity = '0';
      document.body.appendChild(field);
      field.select();
      const copied = document.execCommand('copy');
      field.remove();
      if (!copied) throw new Error('Clipboard copy is unavailable in this browser.');
    };

    document.querySelectorAll('[data-copy]').forEach(button => {
      button.addEventListener('click', async () => {
        const value = button.getAttribute('data-copy') || '';
        const original = button.textContent;
        try {
          await copyText(value.replace(/&#10;/g, '\n'));
          button.textContent = 'Copied';
          button.dataset.copied = 'true';
        } catch (error) {
          button.textContent = 'Copy unavailable';
          button.title = error instanceof Error ? error.message : 'Clipboard unavailable';
        }
        window.setTimeout(() => {
          button.textContent = original;
          button.removeAttribute('data-copied');
        }, 1800);
      });
    });
  };

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', ready, { once: true });
  else ready();
})();
