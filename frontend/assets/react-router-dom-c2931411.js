import{r as l}from"./react-5a14d1f0.js";import{R as k,u as S,a as C,b as L,c as O}from"./react-router-b1e149b2.js";import{c as P,b as m}from"./@remix-run-963a0ed9.js";/**
 * React Router DOM v6.6.1
 *
 * Copyright (c) Remix Software Inc.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE.md file in the root directory of this source tree.
 *
 * @license MIT
 */function h(){return h=Object.assign?Object.assign.bind():function(e){for(var t=1;t<arguments.length;t++){var o=arguments[t];for(var r in o)Object.prototype.hasOwnProperty.call(o,r)&&(e[r]=o[r])}return e},h.apply(this,arguments)}function w(e,t){if(e==null)return{};var o={},r=Object.keys(e),a,n;for(n=0;n<r.length;n++)a=r[n],!(t.indexOf(a)>=0)&&(o[a]=e[a]);return o}function j(e){return!!(e.metaKey||e.altKey||e.ctrlKey||e.shiftKey)}function x(e,t){return e.button===0&&(!t||t==="_self")&&!j(e)}const E=["onClick","relative","reloadDocument","replace","state","target","to","preventScrollReset"];function D(e){let{basename:t,children:o,window:r}=e,a=l.useRef();a.current==null&&(a.current=P({window:r,v5Compat:!0}));let n=a.current,[i,s]=l.useState({action:n.action,location:n.location});return l.useLayoutEffect(()=>n.listen(s),[n]),l.createElement(k,{basename:t,children:o,location:i.location,navigationType:i.action,navigator:n})}const H=l.forwardRef(function(t,o){let{onClick:r,relative:a,reloadDocument:n,replace:i,state:s,target:c,to:u,preventScrollReset:f}=t,p=w(t,E),g=S(u,{relative:a}),R=K(u,{replace:i,state:s,target:c,preventScrollReset:f,relative:a});function y(d){r&&r(d),d.defaultPrevented||R(d)}return l.createElement("a",h({},p,{href:g,onClick:n?r:y,ref:o,target:c}))});var v;(function(e){e.UseScrollRestoration="useScrollRestoration",e.UseSubmitImpl="useSubmitImpl",e.UseFetcher="useFetcher"})(v||(v={}));var b;(function(e){e.UseFetchers="useFetchers",e.UseScrollRestoration="useScrollRestoration"})(b||(b={}));function K(e,t){let{target:o,replace:r,state:a,preventScrollReset:n,relative:i}=t===void 0?{}:t,s=C(),c=L(),u=O(e,{relative:i});return l.useCallback(f=>{if(x(f,o)){f.preventDefault();let p=r!==void 0?r:m(c)===m(u);s(e,{replace:p,state:a,preventScrollReset:n,relative:i})}},[c,s,u,r,a,o,e,n,i])}export{D as B,H as L};
