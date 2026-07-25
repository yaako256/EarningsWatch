export class ApiError extends Error { code: string; status: number; constructor(code:string, message:string, status:number) { super(message); this.code = code; this.status = status } }
type Envelope<T> = { data:T|null; error:{code:string;message:string}|null }
let refreshing: Promise<boolean>|undefined
export const api = {
  onUnauthorized: undefined as undefined | (()=>void),
  async request<T>(path:string, init:RequestInit={}, retried=false):Promise<T>{
    const response=await fetch('/api'+path,{...init,credentials:'include',headers:{...(init.body?{'Content-Type':'application/json'}:{}),...init.headers}})
    const body=await response.json() as Envelope<T>
    if(response.status===401 && !retried && path!=='/auth/refresh') { refreshing ??= fetch('/api/auth/refresh',{method:'POST',credentials:'include'}).then(async r=>r.ok && !(await r.json() as Envelope<null>).error).catch(()=>false).finally(()=>{refreshing=undefined}); if(await refreshing)return this.request<T>(path,init,true); this.onUnauthorized?.() }
    if(!response.ok || body.error) throw new ApiError(body.error?.code??'internal_error',body.error?.message??'リクエストに失敗しました。',response.status)
    return body.data as T
  },
  get<T>(path:string){return this.request<T>(path)}, post<T>(path:string, body?:unknown){return this.request<T>(path,{method:'POST',body:body===undefined?undefined:JSON.stringify(body)})}, put<T>(path:string,body:unknown){return this.request<T>(path,{method:'PUT',body:JSON.stringify(body)})}, patch<T>(path:string,body?:unknown){return this.request<T>(path,{method:'PATCH',body:body===undefined?undefined:JSON.stringify(body)})}, del<T>(path:string){return this.request<T>(path,{method:'DELETE'})}, login(username:string,password:string){return this.post<{username:string;role:'admin'|'user'}>('/auth/login',{username,password})}
}
export const qs=(obj:Record<string,unknown>)=>new URLSearchParams(Object.entries(obj).filter(([,v])=>v!==undefined&&v!==null&&v!=='').map(([k,v])=>[k,String(v)])).toString()
export async function download(path:string,name:string){const response=await fetch('/api'+path,{credentials:'include'});if(!response.ok)throw new Error('ダウンロードに失敗しました。');const a=document.createElement('a');a.href=URL.createObjectURL(await response.blob());a.download=name;a.click();URL.revokeObjectURL(a.href)}
