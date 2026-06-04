window.__splashStart=Date.now();
// Apply saved theme immediately to avoid flash of wrong theme
try{var t=localStorage.getItem('pebble-theme')||'light';var d=t==='system'?matchMedia('(prefers-color-scheme:dark)').matches?'dark':'light':t;document.documentElement.setAttribute('data-theme',d)}catch(e){}
// splash 由 HTML 自主管理生命周期，React 只发送"应用已就绪"信号。
// 这样即使 React 未挂载或 transitionend 丢失，也不会永久遮挡 UI。
(function(){
  var minMs = 900;
  var maxMs = 6000;
  var removeFallbackMs = 700;
  var dismissed = false;
  var removed = false;
  var removeTimer = 0;
  var start = window.__splashStart || Date.now();
  var dispatchRemoved = function(reason){
    try {
      window.dispatchEvent(new CustomEvent('pebble:splash-removed', {
        detail: { reason: reason || 'unknown', elapsedMs: Date.now() - start }
      }));
    } catch(e) {}
  };
  var remove = function(reason){
    if (removed) return;
    removed = true;
    if (removeTimer) window.clearTimeout(removeTimer);
    var s = document.getElementById('splash');
    if (s && s.parentNode) s.parentNode.removeChild(s);
    var st = document.getElementById('splash-style');
    if (st && st.parentNode) st.parentNode.removeChild(st);
    dispatchRemoved(reason);
  };
  var fade = function(reason){
    var s = document.getElementById('splash');
    if (!s) { remove(reason); return; }
    var onTransitionEnd = function(event){
      if (event.target !== s || event.propertyName !== 'opacity') return;
      remove(reason);
    };
    s.addEventListener('transitionend', onTransitionEnd);
    s.classList.add('fade-out');
    removeTimer = window.setTimeout(function(){
      s.removeEventListener('transitionend', onTransitionEnd);
      remove(reason);
    }, removeFallbackMs);
  };
  var dismiss = function(reason){
    if (dismissed) return;
    dismissed = true;
    var elapsed = Date.now() - start;
    if (elapsed >= minMs) { fade(reason); return; }
    window.setTimeout(function(){ fade(reason); }, minMs - elapsed);
  };
  window.pebbleSplash = {
    dismiss: dismiss,
    isDismissed: function(){ return dismissed; },
    isRemoved: function(){ return removed; }
  };
  window.setTimeout(function(){ dismiss('timeout'); }, maxMs);
})();
