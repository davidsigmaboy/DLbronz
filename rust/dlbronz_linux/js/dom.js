(function (g) {
'use strict';
var VOID = {area:1,base:1,br:1,col:1,embed:1,hr:1,img:1,input:1,link:1,meta:1,param:1,source:1,track:1,wbr:1};
var RAW = {script:1,style:1};
function esc(s){return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');}
function escAttr(s){return String(s).replace(/&/g,'&amp;').replace(/"/g,'&quot;');}
function kebab(p){return String(p).replace(/[A-Z]/g,function(c){return '-'+c.toLowerCase();});}
function camel(p){return String(p).replace(/-([a-z])/g,function(_,c){return c.toUpperCase();});}

// ───────── events ─────────
class Ev {
  constructor(t,i){ i=i||{}; this.type=t; this.bubbles=!!i.bubbles; this.cancelable=!!i.cancelable; this.detail=i.detail; this.defaultPrevented=false; this.target=null; this.timeStamp=0; }
  preventDefault(){ this.defaultPrevented=true; }
  stopPropagation(){ this._stop=true; }
  stopImmediatePropagation(){ this._stop=true; }
}
class EventTarget_ {
  constructor(){ this._l={}; }
  addEventListener(t,f){ (this._l[t]=this._l[t]||[]).push(f); }
  removeEventListener(t,f){ var a=this._l[t]; if(a){ var i=a.indexOf(f); if(i>=0) a.splice(i,1); } }
  dispatchEvent(e){
    if(!e.target) e.target=this; e.currentTarget=this;
    var a=(this._l[e.type]||[]).slice();
    for(var i=0;i<a.length;i++){ try{ if(typeof a[i]==='function') a[i].call(this,e); else if(a[i]&&a[i].handleEvent) a[i].handleEvent(e); }catch(err){ __log('error',String(err)); } }
    var h=this['on'+e.type]; if(typeof h==='function'){ try{ h.call(this,e); }catch(err){ __log('error',String(err)); } }
    if(e.bubbles && !e._stop){ if(this.parentNode) this.parentNode.dispatchEvent(e); else if(this===document) g.dispatchEvent(e); }
    return !e.defaultPrevented;
  }
}

// ───────── selectors ─────────
function parseOne(s){
  var re=/\s*([>+~])\s*|(\s+)|(\*)|([A-Za-z0-9_-]+)|#([\w-]+)|\.([\w-]+)|\[([^\]]+)\]|:([\w-]+)(\(([^)]*)\))?/g, m, parts=[], cur=null, comb=' ', last=0;
  s=s.trim();
  while((m=re.exec(s))){
    if(m.index!==last) return null; last=re.lastIndex;
    if(m[1]){ if(cur){parts.push(cur);cur=null;} comb=m[1]; }
    else if(m[2]){ if(cur){parts.push(cur);cur=null;} comb=' '; }
    else {
      if(!cur) cur={comb:comb,tag:null,id:null,cls:[],attrs:[],pseudo:[]};
      if(m[3]){}
      else if(m[4]) cur.tag=m[4].toLowerCase();
      else if(m[5]) cur.id=m[5];
      else if(m[6]) cur.cls.push(m[6]);
      else if(m[7]){ var am=/^([\w-]+)\s*(?:([~|^$*]?=)\s*["']?([^"']*)["']?)?$/.exec(m[7].trim()); if(!am) return null; cur.attrs.push({n:am[1].toLowerCase(),op:am[2]||null,v:am[3]}); }
      else if(m[8]) cur.pseudo.push({n:m[8],a:m[10]});
    }
  }
  if(last!==s.length) return null;
  if(cur) parts.push(cur);
  return parts.length?parts:null;
}
var selCache={};
function parseSel(s){
  if(selCache[s]) return selCache[s];
  var out=s.split(',').map(parseOne).filter(Boolean);
  return (selCache[s]=out);
}
function prevEl(el){ var p=el.parentNode; if(!p) return null; var i=p.childNodes.indexOf(el); for(i--;i>=0;i--) if(p.childNodes[i].nodeType===1) return p.childNodes[i]; return null; }
function matchCompound(el,c){
  if(c.tag && el.localName!==c.tag) return false;
  if(c.id && el._a.id!==c.id) return false;
  if(c.cls.length){ var cl=(el._a['class']||'').split(/\s+/); for(var i=0;i<c.cls.length;i++) if(cl.indexOf(c.cls[i])<0) return false; }
  for(var j=0;j<c.attrs.length;j++){
    var a=c.attrs[j], v=el._a[a.n];
    if(v===undefined) return false;
    switch(a.op){
      case '=': if(v!==a.v) return false; break;
      case '~=': if(v.split(/\s+/).indexOf(a.v)<0) return false; break;
      case '^=': if(v.indexOf(a.v)!==0) return false; break;
      case '$=': if(v.slice(-a.v.length)!==a.v) return false; break;
      case '*=': if(v.indexOf(a.v)<0) return false; break;
      case '|=': if(v!==a.v && v.indexOf(a.v+'-')!==0) return false; break;
    }
  }
  for(var k=0;k<c.pseudo.length;k++){
    var p=c.pseudo[k], par=el.parentNode, sibs=par?par.children:[el], idx=sibs.indexOf(el);
    switch(p.n){
      case 'first-child': if(idx!==0) return false; break;
      case 'last-child': if(idx!==sibs.length-1) return false; break;
      case 'only-child': if(sibs.length!==1) return false; break;
      case 'root': if(el!==document.documentElement) return false; break;
      case 'empty': if(el.childNodes.length) return false; break;
      case 'checked': if(el._a.checked===undefined) return false; break;
      case 'disabled': if(el._a.disabled===undefined) return false; break;
      case 'not': if(matchesSel(el,p.a||'')) return false; break;
      case 'nth-child': {
        var n=(p.a||'').trim(), ok=false;
        if(n==='odd') ok=(idx%2===0); else if(n==='even') ok=(idx%2===1); else if(/^\d+$/.test(n)) ok=(idx+1===+n); else { var f=/^(-?\d*)n\s*([+-]\s*\d+)?$/.exec(n); if(f){ var A=f[1]===''?1:(f[1]==='-'?-1:+f[1]), B=f[2]?+f[2].replace(/\s+/g,''):0; var d=idx+1-B; ok = A===0 ? d===0 : (d/A>=0 && d%A===0); } }
        if(!ok) return false; break; }
      default: return false;
    }
  }
  return true;
}
function matchParts(el,parts,i){
  var p=parts[i]; if(!matchCompound(el,p)) return false; if(i===0) return true;
  var s;
  switch(p.comb){
    case '>': return !!el.parentElement && matchParts(el.parentElement,parts,i-1);
    case ' ': for(var a=el.parentElement;a;a=a.parentElement) if(matchParts(a,parts,i-1)) return true; return false;
    case '+': s=prevEl(el); return !!s && matchParts(s,parts,i-1);
    case '~': for(s=prevEl(el);s;s=prevEl(s)) if(matchParts(s,parts,i-1)) return true; return false;
  }
  return false;
}
function matchesSel(el,sel){ var alts=parseSel(sel); for(var i=0;i<alts.length;i++) if(matchParts(el,alts[i],alts[i].length-1)) return true; return false; }
function qsa(root,sel,first){
  var alts=parseSel(sel), out=[], stack=[], i;
  for(i=root.childNodes.length-1;i>=0;i--) stack.push(root.childNodes[i]);
  while(stack.length){
    var n=stack.pop();
    if(n.nodeType!==1) continue;
    for(var k=0;k<alts.length;k++){ if(matchParts(n,alts[k],alts[k].length-1)){ out.push(n); if(first) return out; break; } }
    for(i=n.childNodes.length-1;i>=0;i--) stack.push(n.childNodes[i]);
  }
  return out;
}

// ───────── nodes ─────────
function ser(n){
  switch(n.nodeType){
    case 3: { var p=n.parentNode; return (p&&p.nodeType===1&&RAW[p.localName])?n.data:esc(n.data); }
    case 1: { var t=n.localName, a=''; for(var k in n._a) a+=' '+k+'="'+escAttr(n._a[k])+'"'; if(VOID[t]) return '<'+t+a+'>'; return '<'+t+a+'>'+n.childNodes.map(ser).join('')+'</'+t+'>'; }
    case 9: case 11: return n.childNodes.map(ser).join('');
    default: return '';
  }
}
class NodeBase extends EventTarget_ {
  constructor(){ super(); this.parentNode=null; this.childNodes=[]; }
  get firstChild(){ return this.childNodes[0]||null; }
  get lastChild(){ return this.childNodes[this.childNodes.length-1]||null; }
  get nextSibling(){ var p=this.parentNode; if(!p) return null; return p.childNodes[p.childNodes.indexOf(this)+1]||null; }
  get previousSibling(){ var p=this.parentNode; if(!p) return null; return p.childNodes[p.childNodes.indexOf(this)-1]||null; }
  get parentElement(){ return this.parentNode&&this.parentNode.nodeType===1?this.parentNode:null; }
  get ownerDocument(){ return document; }
  get isConnected(){ var n=this; while(n){ if(n===document) return true; n=n.parentNode; } return false; }
  hasChildNodes(){ return this.childNodes.length>0; }
  appendChild(c){ return this.insertBefore(c,null); }
  insertBefore(c,ref){
    if(c.nodeType===11){ var kids=c.childNodes.slice(); for(var i=0;i<kids.length;i++) this.insertBefore(kids[i],ref); return c; }
    if(c.parentNode) c.parentNode.removeChild(c);
    c.parentNode=this;
    var idx=ref?this.childNodes.indexOf(ref):-1;
    if(idx<0) this.childNodes.push(c); else this.childNodes.splice(idx,0,c);
    return c;
  }
  removeChild(c){ var i=this.childNodes.indexOf(c); if(i>=0){ this.childNodes.splice(i,1); c.parentNode=null; } return c; }
  replaceChild(n,o){ this.insertBefore(n,o); this.removeChild(o); return o; }
  contains(o){ while(o){ if(o===this) return true; o=o.parentNode; } return false; }
  remove(){ if(this.parentNode) this.parentNode.removeChild(this); }
  _toNode(x){ return (x&&typeof x==='object')?x:document.createTextNode(String(x)); }
  append(){ for(var i=0;i<arguments.length;i++) this.appendChild(this._toNode(arguments[i])); }
  prepend(){ var f=this.firstChild; for(var i=0;i<arguments.length;i++) this.insertBefore(this._toNode(arguments[i]),f); }
  before(){ var p=this.parentNode; if(!p) return; for(var i=0;i<arguments.length;i++) p.insertBefore(this._toNode(arguments[i]),this); }
  after(){ var p=this.parentNode; if(!p) return; var n=this.nextSibling; for(var i=0;i<arguments.length;i++) p.insertBefore(this._toNode(arguments[i]),n); }
  replaceWith(x){ if(this.parentNode){ this.parentNode.insertBefore(this._toNode(x),this); this.remove(); } }
  replaceChildren(){ this.childNodes.slice().forEach(function(c){c.parentNode=null;}); this.childNodes=[]; this.append.apply(this,arguments); }
  get textContent(){ return this.childNodes.map(function(c){return c.nodeType===8?'':c.textContent;}).join(''); }
  set textContent(v){ this.childNodes.slice().forEach(function(c){c.parentNode=null;}); this.childNodes=[]; v=String(v==null?'':v); if(v) this.appendChild(document.createTextNode(v)); }
  get children(){ return this.childNodes.filter(function(n){return n.nodeType===1;}); }
  get childElementCount(){ return this.children.length; }
  get firstElementChild(){ return this.children[0]||null; }
  get lastElementChild(){ var c=this.children; return c[c.length-1]||null; }
  get nextElementSibling(){ var p=this.parentNode; if(!p) return null; var i=p.childNodes.indexOf(this); for(i++;i<p.childNodes.length;i++) if(p.childNodes[i].nodeType===1) return p.childNodes[i]; return null; }
  get previousElementSibling(){ return prevEl(this); }
  querySelector(s){ return qsa(this,s,true)[0]||null; }
  querySelectorAll(s){ return qsa(this,s,false); }
  getElementById(id){ return qsa(this,'#'+id,true)[0]||null; }
  getElementsByTagName(t){ t=String(t).toLowerCase(); return t==='*'?qsa(this,'*'):qsa(this,t); }
  getElementsByClassName(c){ return qsa(this,'.'+String(c).trim().split(/\s+/).join('.')); }
  cloneNode(deep){ return cloneOf(this,deep); }
}
class TextNode extends NodeBase {
  constructor(d){ super(); this.data=String(d); this.nodeType=3; this.nodeName='#text'; }
  get textContent(){ return this.data; } set textContent(v){ this.data=String(v); }
  get nodeValue(){ return this.data; } set nodeValue(v){ this.data=String(v); }
  get length(){ return this.data.length; }
}
class Frag extends NodeBase { constructor(){ super(); this.nodeType=11; this.nodeName='#document-fragment'; } }
class El extends NodeBase {
  constructor(tag){ super(); this.nodeType=1; this.localName=String(tag).toLowerCase(); this.tagName=this.localName.toUpperCase(); this.nodeName=this.tagName; this._a={}; }
  getAttribute(n){ n=String(n).toLowerCase(); return (n in this._a)?this._a[n]:null; }
  setAttribute(n,v){ this._a[String(n).toLowerCase()]=String(v); }
  removeAttribute(n){ delete this._a[String(n).toLowerCase()]; }
  hasAttribute(n){ return String(n).toLowerCase() in this._a; }
  toggleAttribute(n,f){ var h=this.hasAttribute(n); if(f===undefined) f=!h; if(f&&!h) this.setAttribute(n,''); else if(!f&&h) this.removeAttribute(n); return f; }
  get attributes(){ var a=this._a; return Object.keys(a).map(function(k){return {name:k,value:a[k]};}); }
  get id(){ return this._a.id||''; } set id(v){ this._a.id=String(v); }
  get className(){ return this._a['class']||''; } set className(v){ this._a['class']=String(v); }
  get href(){ return this._a.href||''; } set href(v){ this._a.href=String(v); }
  get src(){ return this._a.src||''; } set src(v){ this._a.src=String(v); }
  get title(){ return this._a.title||''; } set title(v){ this._a.title=String(v); }
  get value(){ return this._a.value||''; } set value(v){ this._a.value=String(v); }
  get hidden(){ return 'hidden' in this._a; } set hidden(v){ if(v) this._a.hidden=''; else delete this._a.hidden; }
  _cls(){ return this.className.split(/\s+/).filter(Boolean); }
  get classList(){
    var el=this;
    return {
      add:function(){ var s=el._cls(); for(var i=0;i<arguments.length;i++) if(s.indexOf(arguments[i])<0) s.push(arguments[i]); el.className=s.join(' '); },
      remove:function(){ var s=el._cls(), r=[].slice.call(arguments); el.className=s.filter(function(x){return r.indexOf(x)<0;}).join(' '); },
      contains:function(c){ return el._cls().indexOf(c)>=0; },
      toggle:function(c,f){ var h=el._cls().indexOf(c)>=0; if(f===undefined) f=!h; if(f&&!h) this.add(c); else if(!f&&h) this.remove(c); return f; },
      get length(){ return el._cls().length; }
    };
  }
  get dataset(){
    var el=this;
    return new Proxy({},{
      get:function(t,p){ var v=el._a['data-'+kebab(p)]; return v===undefined?undefined:v; },
      set:function(t,p,v){ el._a['data-'+kebab(p)]=String(v); return true; }
    });
  }
  get style(){
    var el=this;
    function map(){ var m={}; (el._a.style||'').split(';').forEach(function(p){ var i=p.indexOf(':'); if(i>0) m[p.slice(0,i).trim()]=p.slice(i+1).trim(); }); return m; }
    function save(m){ el._a.style=Object.keys(m).map(function(k){return k+': '+m[k];}).join('; '); }
    return new Proxy({},{
      get:function(t,p){
        if(p==='setProperty') return function(k,v){ var m=map(); m[k]=String(v); save(m); };
        if(p==='getPropertyValue') return function(k){ return map()[k]||''; };
        if(p==='removeProperty') return function(k){ var m=map(); delete m[k]; save(m); };
        if(p==='cssText') return el._a.style||'';
        if(typeof p!=='string') return undefined;
        return map()[kebab(p)]||'';
      },
      set:function(t,p,v){ if(p==='cssText'){ el._a.style=String(v); return true; } var m=map(); m[kebab(p)]=String(v); save(m); return true; }
    });
  }
  get innerHTML(){ return this.childNodes.map(ser).join(''); }
  set innerHTML(h){ this.childNodes.slice().forEach(function(c){c.parentNode=null;}); this.childNodes=[]; this.appendChild(parseFragment(String(h))); }
  get outerHTML(){ return ser(this); }
  get innerText(){ return this.textContent; } set innerText(v){ this.textContent=v; }
  insertAdjacentHTML(pos,h){
    var f=parseFragment(String(h));
    switch(String(pos).toLowerCase()){
      case 'beforebegin': if(this.parentNode) this.parentNode.insertBefore(f,this); break;
      case 'afterbegin': this.insertBefore(f,this.firstChild); break;
      case 'beforeend': this.appendChild(f); break;
      case 'afterend': if(this.parentNode) this.parentNode.insertBefore(f,this.nextSibling); break;
    }
  }
  matches(s){ return matchesSel(this,s); }
  closest(s){ for(var n=this;n&&n.nodeType===1;n=n.parentNode) if(matchesSel(n,s)) return n; return null; }
  getBoundingClientRect(){ return {x:0,y:0,top:0,left:0,right:0,bottom:0,width:0,height:0}; }
  getClientRects(){ return []; }
  focus(){} blur(){} scrollIntoView(){} scrollTo(){}
  click(){ this.dispatchEvent(new Ev('click',{bubbles:true,cancelable:true})); }
  get offsetWidth(){ return 0; } get offsetHeight(){ return 0; } get clientWidth(){ return 0; } get clientHeight(){ return 0; }
  get scrollTop(){ return 0; } set scrollTop(v){}
  get nextSiblingElement(){ return this.nextElementSibling; }
}
function cloneOf(n,deep){
  if(n.nodeType===3) return new TextNode(n.data);
  if(n.nodeType===1){ var e=new El(n.localName); for(var k in n._a) e._a[k]=n._a[k]; if(deep) n.childNodes.forEach(function(c){ e.appendChild(cloneOf(c,true)); }); return e; }
  var f=new Frag(); if(deep) n.childNodes.forEach(function(c){ f.appendChild(cloneOf(c,true)); }); return f;
}

class Doc extends NodeBase {
  constructor(){ super(); this.nodeType=9; this.nodeName='#document'; this.readyState='loading'; this.cookie=''; this.referrer=''; this.currentScript=null; this.defaultView=g; this.characterSet='UTF-8'; this.compatMode='CSS1Compat'; this.contentType='text/html'; }
  createElement(t){ return new El(t); }
  createElementNS(ns,t){ return new El(t); }
  createTextNode(d){ return new TextNode(d); }
  createComment(d){ return new TextNode(''); }
  createDocumentFragment(){ return new Frag(); }
  createEvent(){ return new Ev(''); }
  get documentElement(){ return this.children[0]||null; }
  get head(){ var h=this.documentElement; return h?(h.children.filter(function(c){return c.localName==='head';})[0]||null):null; }
  get body(){ var h=this.documentElement; return h?(h.children.filter(function(c){return c.localName==='body';})[0]||null):null; }
  get title(){ var t=this.querySelector('title'); return t?t.textContent.trim():''; }
  set title(v){ var t=this.querySelector('title'); if(!t){ t=this.createElement('title'); if(this.head) this.head.appendChild(t); } t.textContent=v; }
  get URL(){ return g.location.href; } get documentURI(){ return g.location.href; }
  get domain(){ return g.location.hostname; }
  get activeElement(){ return this.body; }
  hasFocus(){ return true; }
  write(){} writeln(){} open(){} close(){}
  get scripts(){ return qsa(this,'script'); }
  get links(){ return qsa(this,'a[href]'); }
  get images(){ return qsa(this,'img'); }
  get forms(){ return qsa(this,'form'); }
}

var document=new Doc();
function build(j){
  if(j.t==='x') return new TextNode(j.v);
  var el=new El(j.n);
  for(var k in j.a) el._a[k]=j.a[k];
  (j.c||[]).forEach(function(c){ el.appendChild(build(c)); });
  return el;
}
function parseFragment(h){
  var j=JSON.parse(__parseFragment(h)), f=new Frag();
  (j.c||[]).forEach(function(c){ f.appendChild(build(c)); });
  return f;
}
(JSON.parse(__domJSON).c||[]).forEach(function(c){ document.appendChild(build(c)); });

// ───────── window ─────────
var m=/^(https?:)\/\/([^\/?#]*)([^?#]*)(\?[^#]*)?(#.*)?$/.exec(__baseURL)||[];
var loc={
  href:__baseURL, protocol:m[1]||'https:', host:m[2]||'', hostname:(m[2]||'').replace(/:\d+$/,''),
  port:((/:(\d+)$/.exec(m[2]||''))||[])[1]||'', pathname:m[3]||'/', search:m[4]||'', hash:m[5]||'',
  origin:(m[1]||'https:')+'//'+(m[2]||''),
  toString:function(){ return this.href; }, assign:function(){}, replace:function(){}, reload:function(){}
};
g.window=g; g.self=g; g.top=g; g.parent=g; g.frames=g;
g.document=document; g.location=loc;
g.Node=NodeBase; g.Element=El; g.HTMLElement=El; g.Text=TextNode; g.Document=Doc; g.DocumentFragment=Frag;
g.Event=Ev; g.CustomEvent=Ev; g.EventTarget=EventTarget_;
g.HTMLDivElement=El; g.HTMLInputElement=El; g.HTMLAnchorElement=El; g.HTMLImageElement=El; g.HTMLScriptElement=El;
g.navigator={userAgent:'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) DLbronz/0.2', language:'en-US', languages:['en-US','en'], platform:'iPhone', vendor:'', cookieEnabled:false, onLine:true, hardwareConcurrency:2, maxTouchPoints:5, sendBeacon:function(){return false;}};
g.screen={width:390,height:844,availWidth:390,availHeight:844,colorDepth:24,pixelDepth:24};
g.innerWidth=390; g.innerHeight=844; g.outerWidth=390; g.outerHeight=844; g.devicePixelRatio=3; g.scrollX=0; g.scrollY=0; g.pageXOffset=0; g.pageYOffset=0;
g.history={length:1,state:null,pushState:function(){},replaceState:function(){},back:function(){},forward:function(){},go:function(){}};
function Store(){ var d={}; return {getItem:function(k){return (k in d)?d[k]:null;},setItem:function(k,v){d[k]=String(v);},removeItem:function(k){delete d[k];},clear:function(){d={};},key:function(i){return Object.keys(d)[i]||null;},get length(){return Object.keys(d).length;}}; }
g.localStorage=Store(); g.sessionStorage=Store();
var t0=0; g.performance={now:function(){return t0+=1;},mark:function(){},measure:function(){},timing:{},getEntriesByType:function(){return [];},getEntriesByName:function(){return [];}};
g.getComputedStyle=function(el){ return el.style; };
g.matchMedia=function(q){ return {matches:false,media:String(q),addListener:function(){},removeListener:function(){},addEventListener:function(){},removeEventListener:function(){}}; };
g.scrollTo=g.scroll=g.scrollBy=g.focus=g.blur=g.alert=g.open=g.close=function(){};
g.confirm=function(){return false;}; g.prompt=function(){return null;};
function Obs(){ this.observe=function(){}; this.unobserve=function(){}; this.disconnect=function(){}; this.takeRecords=function(){return [];}; }
g.MutationObserver=Obs; g.IntersectionObserver=Obs; g.ResizeObserver=Obs; g.PerformanceObserver=Obs;
g.queueMicrotask=function(f){ Promise.resolve().then(f); };
g.fetch=function(){ return Promise.reject(new Error('fetch is not supported')); };
g.XMLHttpRequest=function(){ this.open=function(){}; this.send=function(){}; this.setRequestHeader=function(){}; this.abort=function(){}; this.readyState=0; this.status=0; };
g.Image=function(){ return document.createElement('img'); };
var B64='ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
g.btoa=function(s){ s=String(s); var o='',i=0,a,b,c; while(i<s.length){ a=s.charCodeAt(i++); b=s.charCodeAt(i++); c=s.charCodeAt(i++); o+=B64[a>>2]+B64[((a&3)<<4)|(b>>4||0)]+(isNaN(b)?'=':B64[((b&15)<<2)|(c>>6||0)])+(isNaN(c)?'=':B64[c&63]); } return o; };
g.atob=function(s){ s=String(s).replace(/=+$/,''); var o='',bits=0,acc=0; for(var i=0;i<s.length;i++){ var v=B64.indexOf(s[i]); if(v<0) continue; acc=(acc<<6)|v; bits+=6; if(bits>=8){ bits-=8; o+=String.fromCharCode((acc>>bits)&255); } } return o; };

// window as an event target
var wl={};
g.addEventListener=function(t,f){ (wl[t]=wl[t]||[]).push(f); };
g.removeEventListener=function(t,f){ var a=wl[t]; if(a){ var i=a.indexOf(f); if(i>=0) a.splice(i,1); } };
g.dispatchEvent=function(e){ e.target=e.target||g; var a=(wl[e.type]||[]).slice(); for(var i=0;i<a.length;i++){ try{ a[i].call(g,e); }catch(err){ __log('error',String(err)); } } var h=g['on'+e.type]; if(typeof h==='function'){ try{ h.call(g,e); }catch(err){ __log('error',String(err)); } } return true; };

// console
function fmt(a){ if(typeof a==='string') return a; try{ return JSON.stringify(a); }catch(e){ return String(a); } }
g.console={};
['log','info','debug','warn','error','trace'].forEach(function(l){ g.console[l]=function(){ __log(l,[].map.call(arguments,fmt).join(' ')); }; });
['group','groupEnd','groupCollapsed','table','time','timeEnd','dir','assert','count','clear'].forEach(function(l){ g.console[l]=function(){}; });

// timers (virtual time, run after the scripts)
var timers=[], tid=1, vnow=0;
g.setTimeout=function(f,d){ var id=tid++; timers.push({id:id,f:f,t:vnow+(+d||0),seq:id,args:[].slice.call(arguments,2)}); return id; };
g.setInterval=function(f,d){ var id=tid++; timers.push({id:id,f:f,t:vnow+(+d||0),seq:id,iv:Math.max(+d||0,1),args:[].slice.call(arguments,2)}); return id; };
g.clearTimeout=g.clearInterval=function(id){ timers=timers.filter(function(t){return t.id!==id;}); };
g.requestAnimationFrame=function(f){ return g.setTimeout(function(){ f(vnow); },16); };
g.cancelAnimationFrame=g.clearTimeout;
g.requestIdleCallback=function(f){ return g.setTimeout(function(){ f({didTimeout:false,timeRemaining:function(){return 10;}}); },1); };
g.__runTimers=function(limit){
  var n=0;
  while(timers.length&&n<limit){
    timers.sort(function(a,b){return a.t-b.t||a.seq-b.seq;});
    var t=timers.shift(); vnow=Math.max(vnow,t.t);
    if(t.iv){ t.runs=(t.runs||0)+1; if(t.runs<3){ t.t=vnow+t.iv; t.seq=tid++; timers.push(t); } }
    try{ if(typeof t.f==='function') t.f.apply(g,t.args); }catch(e){ __log('error',String(e)); }
    n++;
  }
};

// ───────── hooks used by Swift ─────────
var scriptNodes=[];
g.__scripts=function(){
  var out=[]; scriptNodes=[];
  qsa(document,'script').forEach(function(s){
    var ty=(s._a.type||'').toLowerCase();
    if(ty&&ty!=='text/javascript'&&ty!=='application/javascript'&&ty!=='text/ecmascript') return;
    if('nomodule' in s._a) return;
    scriptNodes.push(s);
    out.push({index:scriptNodes.length-1,src:s._a.src||null});
  });
  return JSON.stringify(out);
};
g.__scriptText=function(i){ return scriptNodes[i]?scriptNodes[i].textContent:''; };
g.__setCurrent=function(i){ document.currentScript=scriptNodes[i]||null; };
g.__stylesheets=function(){
  return JSON.stringify(qsa(document,'link').filter(function(l){
    var rel=(l._a.rel||'').toLowerCase(), media=(l._a.media||'').toLowerCase();
    return rel.indexOf('stylesheet')>=0 && l._a.href && media!=='print';
  }).map(function(l){ return l._a.href; }));
};
g.__fire=function(){
  document.readyState='interactive';
  document.dispatchEvent(new Ev('readystatechange'));
  document.dispatchEvent(new Ev('DOMContentLoaded',{bubbles:true}));
  document.readyState='complete';
  document.dispatchEvent(new Ev('readystatechange'));
  g.dispatchEvent(new Ev('load'));
};
g.__serialize=function(){ var h=document.documentElement; return h?ser(h):''; };
})(this);
