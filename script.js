/* Karl Mark Linux — original cursor-swarm page reveal.
   The motion is independently implemented; it does not import the Swarm Rebuild source. */
(()=>{
 const canvas=document.getElementById("swarmCanvas"), intro=document.getElementById("buildIntro");
 if(!canvas||!intro)return;
 const ctx=canvas.getContext("2d",{alpha:true});
 const reduced=window.matchMedia("(prefers-reduced-motion: reduce)").matches;
 const skip=document.getElementById("skipIntro");
 let w=0,h=0,dpr=1,raf=0,started=performance.now(),done=false;
 const cursors=[], trails=[];
 const rand=(a,b)=>a+Math.random()*(b-a);
 function resize(){dpr=Math.min(devicePixelRatio||1,2);w=innerWidth;h=innerHeight;canvas.width=w*dpr;canvas.height=h*dpr;canvas.style.width=w+"px";canvas.style.height=h+"px";ctx.setTransform(dpr,0,0,dpr,0,0)}
 resize();addEventListener("resize",resize);
 const targets=[];
 function addTarget(el,delay=0){
   if(!el)return;
   const r=el.getBoundingClientRect();
   if(r.width<2||r.height<2)return;
   targets.push({x:r.left+r.width/2,y:r.top+r.height/2,el,delay});
 }
 // Each cursor assembles a real page region, then the page is revealed.
 const selectors=[".brand",".top-center",".top-right",".hero-copy .eyebrow",".hero-copy h1",".hero-sub",".hero-actions",".hero-meta",".desktop-stage",".below-fold",".feature-grid","footer"];
 selectors.forEach((s,i)=>addTarget(document.querySelector(s),i*.17));
 if(!targets.length){finish();return}
 const colors=["#d8c4ff","#f0b8d0","#a9d9c1","#f0c98f","#c4b6f4"];
 const count=Math.min(24,Math.max(12,Math.floor((w*h)/65000)));
 for(let i=0;i<count;i++){
   const t=targets[i%targets.length];
   cursors.push({x:rand(0,w),y:rand(0,h),tx:t.x+rand(-40,40),ty:t.y+rand(-22,22),speed:rand(.045,.095),phase:rand(0,Math.PI*2),color:colors[i%colors.length],size:rand(3,5),target:i%targets.length});
 }
 function cursorShape(x,y,size,color,angle){
   ctx.save();ctx.translate(x,y);ctx.rotate(angle);
   ctx.fillStyle=color;ctx.shadowColor=color;ctx.shadowBlur=10;
   ctx.beginPath();ctx.moveTo(0,0);ctx.lineTo(0,size*4.5);ctx.lineTo(size*1.25,size*3.15);ctx.lineTo(size*2.2,size*5.1);ctx.lineTo(size*3.1,size*4.55);ctx.lineTo(size*2.15,size*2.65);ctx.lineTo(size*4,size*2.35);ctx.closePath();ctx.fill();
   ctx.restore();
 }
 function finish(){
   if(done)return;done=true;cancelAnimationFrame(raf);
   document.body.classList.remove("is-building");document.body.classList.add("is-built");
   intro.classList.add("done");
   setTimeout(()=>{canvas.style.display="none";intro.remove()},850);
 }
 function frame(now){
   if(done)return;
   const elapsed=(now-started)/1000;
   ctx.clearRect(0,0,w,h);
   // Swarm cursors converge on page elements in staggered waves.
   cursors.forEach((c,i)=>{
     const target=targets[c.target%targets.length];
     const phaseTime=Math.max(0,elapsed-target.delay);
     const wobble=phaseTime<1.1?Math.sin(elapsed*5+c.phase)*18:Math.sin(elapsed*2+c.phase)*2;
     const tx=target.x+wobble,ty=target.y+Math.cos(elapsed*4+c.phase)*wobble*.35;
     c.x+=(tx-c.x)*c.speed;c.y+=(ty-c.y)*c.speed;
     const dx=tx-c.x,dy=ty-c.y;
     const angle=Math.atan2(dy,dx)+Math.PI/2;
     if(i%3===0){
       ctx.strokeStyle=c.color+"55";ctx.lineWidth=.7;ctx.beginPath();ctx.moveTo(c.x,c.y);ctx.lineTo(c.x-dx*.13,c.y-dy*.13);ctx.stroke();
     }
     cursorShape(c.x,c.y,c.size,c.color,angle);
     // A small sparkle marks each completed assembly point.
     if(Math.hypot(dx,dy)<24){
       ctx.globalAlpha=.3+Math.sin(now*.012+i)*.2;ctx.fillStyle=c.color;ctx.beginPath();ctx.arc(tx,ty,1.5,0,Math.PI*2);ctx.fill();ctx.globalAlpha=1;
     }
   });
   if(elapsed>4.6){finish();return}
   raf=requestAnimationFrame(frame);
 }
 function start(){if(reduced){finish();return}raf=requestAnimationFrame(frame)}
 skip?.addEventListener("click",finish);
 setTimeout(start,450);
})();

const $=(s,root=document)=>root.querySelector(s), $$=(s,root=document)=>[...root.querySelectorAll(s)];
const modalData={
about:{kicker:"THE PROJECT",title:"A Linux project with room to grow.",lead:"Karl Mark Linux is an experimental Arch-based operating system project. Its direction brings together a Wayland desktop, Rust-based tooling and a strong focus on a considered user experience.",tiles:[["Current stage","Early development. Features and release readiness are evolving."],["Foundation","Arch Linux base with a Hyprland / Wayland desktop direction."],["Engineering","Rust is the preferred language for project tools and system components."],["Open development","Follow source code, issues and progress on GitHub."]]},
features:{kicker:"01 / EXPERIENCE",title:"Designed to feel considered.",lead:"The project explores a desktop that balances customization, responsiveness and visual character. These are design goals, not a claim that every planned feature is already complete.",tiles:[["Wayland desktop","Hyprland-centered desktop experience and modern compositor workflow."],["Rust tooling","A preference for efficient, maintainable system utilities."],["Hardware awareness","Hardware inventory and setup planning are areas of active development."],["Compatibility vision","Windows and Android application support remain longer-term research goals."]]},
files:{kicker:"DESKTOP APP",title:"Project files",lead:"This browser preview is a visual concept. Use the repository for the real source, build scripts and project documentation.",tiles:[["Source repository","Browse the complete project source on GitHub."],["Documentation","Read the README and development notes before testing a build."]]},
download:{kicker:"DOWNLOAD CENTER",title:"Try the development build.",lead:"Karl Mark Linux is experimental. Check the repository Releases page for an actual published ISO before downloading. No stable release is claimed by this preview.",download:true},
roadmap:{kicker:"DEVELOPMENT",title:"Built in stages.",lead:"The roadmap should reflect verified repository milestones. Planned work is shown separately from features that are already implemented.",tiles:[["Foundation","Arch-based project structure and early bootable environment."],["Desktop","Continue refining the Hyprland desktop and user experience."],["Installer","A graphical installation experience is a future goal."],["Compatibility","Research application compatibility and isolation architecture."]]}
};
function showModal(key){const d=modalData[key]||modalData.about;let html=`<div class="modal-content"><div class="modal-kicker">${d.kicker}</div><h2 class="modal-title" id="modalTitle">${d.title}</h2><p class="modal-lead">${d.lead}</p>`;
if(d.tiles)html+=`<div class="modal-grid">${d.tiles.map(t=>`<div class="info-tile"><b>${t[0]}</b><p>${t[1]}</p></div>`).join("")}</div>`;
if(d.download)html+=`<div class="download-card"><div class="download-row"><span>Project version</span><b>0.0.1_dev</b></div><div class="download-row"><span>Build label</span><b>1101126_1</b></div><div class="download-row"><span>Target architecture</span><b>x86_64</b></div><div class="download-row"><span>Release channel</span><b>Experimental</b></div><p class="download-note">⚠ Development software may contain bugs and may not be suitable for daily use. Verify the release notes and checksum if an ISO is published.</p><a class="modal-cta" href="https://github.com/karlmarkleef/LinuxOSbykrlmrk/releases" target="_blank" rel="noreferrer">Open GitHub Releases ↗</a></div>`;
html+=`<a class="modal-cta" href="https://github.com/karlmarkleef/LinuxOSbykrlmrk" target="_blank" rel="noreferrer">View source repository ↗</a></div>`;
$("#modalContent").innerHTML=html;$("#overlay").classList.add("open");document.body.style.overflow="hidden"}
function closeModal(){$("#overlay").classList.remove("open");document.body.style.overflow=""}
$$("[data-open]").forEach(b=>b.addEventListener("click",()=>showModal(b.dataset.open)));
$("#modalClose").addEventListener("click",closeModal);$("#overlay").addEventListener("click",e=>{if(e.target.id==="overlay")closeModal()});document.addEventListener("keydown",e=>{if(e.key==="Escape")closeModal();if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==="k"){e.preventDefault();showModal("features")}});
$("#quickSettings").addEventListener("click",()=>showModal("features"));$("#dockSettings").addEventListener("click",()=>showModal("about"));$("#profileBtn").addEventListener("click",()=>showModal("about"));
$("#playBtn").addEventListener("click",e=>{e.currentTarget.textContent=e.currentTarget.textContent==="▶"?"Ⅱ":"▶";toast("Demo player interaction")});
function toast(s){const t=$("#toast");t.textContent=s;t.classList.add("show");setTimeout(()=>t.classList.remove("show"),1800)}
function clock(){const d=new Date();$("#clock").textContent=d.toLocaleTimeString([], {hour:"2-digit",minute:"2-digit"})}clock();setInterval(clock,30000);
// Bring windows forward and allow dragging by their title bars.
let z=10;$$(".window").forEach(w=>{w.addEventListener("pointerdown",()=>{w.style.zIndex=++z;$$(".window").forEach(x=>x.classList.remove("focused"));w.classList.add("focused")});const bar=$(".window-bar",w);let drag=null;bar.addEventListener("pointerdown",e=>{if(e.target.closest("button"))return;const r=w.getBoundingClientRect();drag={x:e.clientX-r.left,y:e.clientY-r.top};bar.setPointerCapture(e.pointerId)});bar.addEventListener("pointermove",e=>{if(!drag)return;const stage=$(".desktop-stage").getBoundingClientRect();w.style.left=Math.max(0,Math.min(stage.width-w.offsetWidth,e.clientX-stage.left-drag.x))+"px";w.style.top=Math.max(42,Math.min(stage.height-w.offsetHeight-55,e.clientY-stage.top-drag.y))+"px";w.style.right="auto";w.style.bottom="auto"});bar.addEventListener("pointerup",()=>drag=null);$(".win-close",w).addEventListener("click",()=>{w.classList.add("minimized");setTimeout(()=>w.classList.remove("minimized"),1700)})});
