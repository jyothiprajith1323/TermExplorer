/**
 * TermExplorer v1.1.0 — Interactive Web Application Scripts
 * Manages install tab switching, clipboard copying, interactive workstation preview,
 * mobile drawer toggle, and keyboard shortcuts filter.
 */

document.addEventListener('DOMContentLoaded', () => {
  // ── 1. Multi-Tab Install Command Switcher ────────────────────────────────────
  const installCmds = {
    shell: 'curl -sSL https://raw.githubusercontent.com/jyothiprajith1323/TermExplorer/master/install.sh | bash',
    cargo: 'cargo install --git https://github.com/jyothiprajith1323/TermExplorer.git',
    aur: 'git clone https://github.com/jyothiprajith1323/TermExplorer.git && cd TermExplorer/packaging/aur && makepkg -si',
    source: 'git clone https://github.com/jyothiprajith1323/TermExplorer.git && cd TermExplorer && cargo build --release',
  };

  const installTabs = document.querySelectorAll('.install-tab-btn');
  const installCmdEl = document.getElementById('install-cmd-text');

  if (installTabs.length && installCmdEl) {
    installTabs.forEach(tab => {
      tab.addEventListener('click', () => {
        installTabs.forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        const key = tab.getAttribute('data-install-type');
        if (installCmds[key]) {
          installCmdEl.textContent = installCmds[key];
        }
      });
    });
  }

  // ── 2. Universal Clipboard Copy Handler ─────────────────────────────────────
  const copyBtns = document.querySelectorAll('.copy-btn');
  copyBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      const targetId = btn.getAttribute('data-clipboard-target');
      let text = '';
      if (targetId) {
        const el = document.getElementById(targetId);
        text = el ? (el.textContent || el.innerText) : '';
      } else {
        text = btn.getAttribute('data-clipboard-text') || '';
      }

      if (!text && installCmdEl) {
        text = installCmdEl.textContent || installCmdEl.innerText;
      }

      if (text) {
        navigator.clipboard.writeText(text.trim()).then(() => {
          const originalHtml = btn.innerHTML;
          btn.classList.add('copied');
          btn.innerHTML = '<span>✓</span> Copied!';
          setTimeout(() => {
            btn.classList.remove('copied');
            btn.innerHTML = originalHtml;
          }, 2000);
        }).catch(() => {
          // Fallback for older browsers
          const textarea = document.createElement('textarea');
          textarea.value = text.trim();
          document.body.appendChild(textarea);
          textarea.select();
          document.execCommand('copy');
          document.body.removeChild(textarea);
          btn.classList.add('copied');
          btn.innerHTML = '<span>✓</span> Copied!';
          setTimeout(() => {
            btn.classList.remove('copied');
            btn.innerHTML = originalHtml;
          }, 2000);
        });
      }
    });
  });

  // ── 3. Interactive Workstation Mockup Tabs ──────────────────────────────────
  const mockupTabs = document.querySelectorAll('.mockup-tab-btn');
  const mockupImage = document.getElementById('workstation-hero-img');
  const mockupCaption = document.getElementById('workstation-caption');

  const views = {
    workstation: {
      src: 'assets/termexplorer-ui.svg',
      caption: '25% File Explorer  │  75% Contextual Preview  │  Embedded Linux PTY Terminal (~20% Height)',
    },
    preview: {
      src: 'assets/termexplorer-ui.svg',
      caption: 'Universal Multi-Format Preview: Source Code, 24-bit Half-Block Images, Hex Inspector, Audio/Video Meta',
    },
    settings: {
      src: 'assets/termexplorer-ui.svg',
      caption: 'Interactive Settings Studio (F10): 14 Categories, Catppuccin Palettes, Icon Packs & Typography',
    },
    help: {
      src: 'assets/termexplorer-ui.svg',
      caption: 'Keyboard Shortcuts Reference Matrix (F1 / ?): Categorized pills, tab navigation & workflow pro tips',
    },
  };

  if (mockupTabs.length && mockupImage) {
    mockupTabs.forEach(tab => {
      tab.addEventListener('click', () => {
        mockupTabs.forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        const viewKey = tab.getAttribute('data-view');
        if (views[viewKey]) {
          mockupImage.src = views[viewKey].src;
          if (mockupCaption) {
            mockupCaption.textContent = views[viewKey].caption;
          }
        }
      });
    });
  }

  // ── 4. Mobile Menu Drawer Toggle ────────────────────────────────────────────
  const mobileBtn = document.querySelector('.mobile-menu-btn');
  const navLinks = document.querySelector('.nav-links');
  if (mobileBtn && navLinks) {
    mobileBtn.addEventListener('click', () => {
      navLinks.classList.toggle('open');
      const isOpen = navLinks.classList.contains('open');
      mobileBtn.setAttribute('aria-expanded', isOpen);
    });
  }

  // ── 5. Real-Time Keyboard Shortcuts Filter (for docs.html) ───────────────────
  const shortcutSearchInput = document.getElementById('shortcut-search-input');
  if (shortcutSearchInput) {
    shortcutSearchInput.addEventListener('input', (e) => {
      const q = e.target.value.toLowerCase().trim();
      const rows = document.querySelectorAll('.shortcut-row');
      rows.forEach(row => {
        const text = row.textContent.toLowerCase();
        row.style.display = text.includes(q) ? '' : 'none';
      });
    });
  }
});
