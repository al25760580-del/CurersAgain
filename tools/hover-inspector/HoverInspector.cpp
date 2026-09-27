#include <Aurie/shared.hpp>
#include <YYToolkit/YYTK_Shared.hpp>
#include <CallbackManager/CallbackManagerInterface.h>
#include <fstream>
#include <sstream>
#include <iomanip>
#include <unordered_map>
#include <cmath>
#include <cstring>
using namespace Aurie;using namespace YYTK;
static YYTKInterface* yy=nullptr;static CallbackManagerInterface* cb=nullptr;static AurieModule* module=nullptr;
static fs::path dir;static std::ofstream logFile;static std::string buffer;static bool active=false,own=false,settingsLayer=false;
static CInstance* scoresSelf=nullptr;static std::string pendingHover;
using Builtin=void(*)(RValue&,CInstance*,CInstance*,int,RValue*);
static Builtin drawStateGetters[5]={};
static std::unordered_map<int,std::string> builtinNames;
static unsigned long long frame=0,total=0;static int remaining=0;static double oldx=-99999,oldy=-99999;
static const std::string mod="HiScores passive hover inspector";
struct SpriteInfo{int id;std::string name;};static std::unordered_map<uintptr_t,SpriteInfo> sprites;
static std::string quote(const std::string& x){std::ostringstream s;s<<'"';for(unsigned char c:x){if(c=='"'||c=='\\')s<<'\\'<<c;else if(c<32)s<<"\\u"<<std::hex<<std::setw(4)<<std::setfill('0')<<unsigned(c)<<std::dec;else s<<c;}s<<'"';return s.str();}
static std::string num(double d){if(!std::isfinite(d))return "null";std::ostringstream s;s<<std::setprecision(17)<<d;return s.str();}
static std::string value(RValue& v){switch(v.m_Kind){case VALUE_REAL:return num(v.m_Real);case VALUE_INT32:return std::to_string(v.m_i32);case VALUE_INT64:return std::to_string(v.m_i64);case VALUE_BOOL:return v.ToBoolean()?"true":"false";case VALUE_STRING:return quote(v.ToString().substr(0,1024));case VALUE_REF:return "{\"asset_id\":"+std::to_string(uint32_t(v.m_i64))+"}";default:return "null";}}
static void emit(const std::string& s){if(buffer.size()<256*1024)buffer+=s+"\n";}
static std::string context(){return "\"frame\":"+std::to_string(frame)+",\"tick_ms\":"+std::to_string(GetTickCount64());}
static std::string drawState(){std::string s;const char* keys[]={"font","color","alpha","halign","valign"};for(int i=0;i<5;i++)if(drawStateGetters[i]){RValue v;drawStateGetters[i](v,nullptr,nullptr,0,nullptr);s+=","+quote(keys[i])+":"+value(v);}return s;}
static void snapshotState(CInstance* self,const char* phase){
  std::string s="{\"event\":\"ui_state\","+context()+",\"phase\":"+quote(phase)+",\"vars\":{";bool first=true;
  for(const char* k:{"currentOption","currentSettingsOption","showOptions","changingName","renameOption","canType","deleteConfirm","deleteOption","deleteSelect","showScores","showScoresIndex","showingAllCharacter","characterOption","currentLegacyLeaderboardVersion","selectingAlpha","showScoresMenuSlideCurrent","showScoresMenuSlideNormalized","canControl","rectTime","rectVis"}){RValue* v=nullptr;if(AurieSuccess(yy->GetInstanceMember(self,k,v))&&v){if(!first)s+=",";first=false;s+=quote(k)+":"+value(*v);}}
  emit(s+"}}");
}
static void beforeDraw(CodeEventArgs& args){
 scoresSelf=std::get<0>(args);
 frame++;active=false;settingsLayer=false;if(own)return;own=true;
 RValue mx,my;yy->GetBuiltin("mouse_x",nullptr,NULL_INDEX,mx);yy->GetBuiltin("mouse_y",nullptr,NULL_INDEX,my);
 double x=mx.ToDouble(),y=my.ToDouble();
 if(x!=oldx||y!=oldy){remaining=12;oldx=x;oldy=y;}
 std::error_code ec;auto request=dir/"HoverInspector-request.txt";
 if(frame%15==0&&fs::exists(request,ec)){fs::remove(request,ec);remaining=120;}
 active=remaining>0||frame%15==0;if(remaining>0)--remaining;
 if(active){
  buffer=std::move(pendingHover);pendingHover.clear();emit("{\"event\":\"frame_begin\","+context()+",\"mouse_x\":"+num(x)+",\"mouse_y\":"+num(y)+"}");
  snapshotState(scoresSelf,"before_draw");
 }
 pendingHover.clear();own=false;
}
static void afterDraw(CodeEventArgs&){
 if(active){snapshotState(scoresSelf,"after_draw");emit("{\"event\":\"frame_end\","+context()+"}");
  if(total+buffer.size()<32*1024*1024){logFile<<buffer;logFile.flush();total+=buffer.size();}
  auto tmp=dir/"HoverInspector-latest.jsonl.tmp";{std::ofstream f(tmp,std::ios::binary);f<<buffer;}MoveFileExW(tmp.c_str(),(dir/"HoverInspector-latest.jsonl").c_str(),MOVEFILE_REPLACE_EXISTING);
 }active=false;
}
static void script(const char* name,int n,RValue** a,RValue* result=nullptr){if(!active||own)return;own=true;
 std::string s="{\"event\":\"script\","+context()+",\"name\":"+quote(name)+",\"args\":[";
 for(int i=0;i<n&&i<16;i++){if(i)s+=",";s+=a[i]?value(*a[i]):"null";}s+="]";
 if(result)s+=",\"result\":"+value(*result);
 if(std::string(name).find("text")!=std::string::npos){s+=drawState();}
 emit(s+"}");own=false;
}
// YYC dispatch signature recovered from native call sites: self, other,
// result, argc, tagged routine index, RValue** arguments. Observation only.
static void dispatchEntry(ProcessorContext64& c){
 if(!active||own)return;int n=(int)c.R9;if(n<0||n>16)return;
 uint64_t id=0;RValue** argv=nullptr;memcpy(&id,(const void*)(c.RSP+0x28),8);memcpy(&argv,(const void*)(c.RSP+0x30),8);
 auto it=builtinNames.find((int)id);if(it==builtinNames.end()||(!argv&&n))return;
 if(it->second=="draw_text_ext_color"||it->second=="draw_text_ext_colour")return;
 script(it->second.c_str(),n,argv);
}
static RValue& text(CInstance*,CInstance*,RValue& r,int n,RValue**a){script("draw_text_scribble",n,a);return r;}
static RValue& textExt(CInstance*,CInstance*,RValue& r,int n,RValue**a){script("draw_text_scribble_ext",n,a);return r;}
static RValue& outline(CInstance*,CInstance*,RValue& r,int n,RValue**a){script("draw_text_outline",n,a);return r;}
static RValue& hover(CInstance* self,CInstance*,RValue& r,int n,RValue**a){
 if(own||self!=scoresSelf||pendingHover.size()>65536)return r;own=true;
 std::string s="{\"event\":\"hover_test\",\"phase\":"+quote(active?"during_draw":"before_draw")+",\"frame\":"+std::to_string(frame+(active?0:1))+",\"tick_ms\":"+std::to_string(GetTickCount64())+",\"args\":[";
 for(int i=0;i<n&&i<16;i++){if(i)s+=",";s+=a[i]?value(*a[i]):"null";}s+="] ,\"result\":"+value(r)+"}";
 if(active)emit(s);else pendingHover+=s+"\n";own=false;return r;
}
static RValue& scribbleElement(CInstance* self,CInstance*,RValue& r,int n,RValue** a){
 if(!active||own||!settingsLayer)return r;own=true;
 std::string s="{\"event\":\"scribble_element\","+context()+",\"args\":[";
 for(int i=0;i<n&&i<8;i++){if(i)s+=",";s+=a[i]?value(*a[i]):"null";}s+="],\"fields\":{";bool first=true;
 for(const char* k:{"__text","__starting_font","__starting_colour","__starting_halign","__starting_valign","__blend_colour","__blend_alpha","__origin_x","__origin_y","__xscale","__yscale","__angle","__scale_to_box_scale","__wrap_max_width","__wrap_max_height","__wrap_max_scale","__line_spacing","__animation_time","__animation_speed","__animation_blink_state","__msdf_border_thickness","__msdf_shadow_alpha"}){RValue* v=nullptr;if(AurieSuccess(yy->GetInstanceMember(self,k,v))&&v){if(!first)s+=",";first=false;s+=quote(k)+":"+value(*v);}}
 RValue* model=nullptr;if(AurieSuccess(yy->GetInstanceMember(self,"__model",model))&&model&&model->m_Kind==VALUE_OBJECT){RValue* fit=nullptr;if(AurieSuccess(yy->GetInstanceMember(*model,"__fit_scale",fit))&&fit){if(!first)s+=",";s+="\"__model_fit_scale\":"+value(*fit);}}
 emit(s+"}}");own=false;return r;
}
static void scribbleEntry(ProcessorContext64& c){
 if(!active||own||!settingsLayer)return;
 int n=(int)c.R9;if(n<0||n>8)return;
 RValue** argv=nullptr;memcpy(&argv,(const void*)(c.RSP+0x28),sizeof argv);
 RValue dummy;scribbleElement((CInstance*)c.RCX,(CInstance*)c.RDX,dummy,n,argv);
}
static float stackFloat(const ProcessorContext64& c,size_t offset){float f;memcpy(&f,(const void*)(c.RSP+offset),4);return f;}
static void engine(ProcessorContext64& c,int mode){
 if(!active||own)return;auto found=sprites.find(c.RCX);if(found==sprites.end())return;own=true;
 if(found->second.name=="hud_optionsmenu")settingsLayer=true;
 std::string s="{\"event\":\"sprite\","+context()+",\"engine\":"+std::to_string(mode)+",\"sprite\":"+quote(found->second.name)+",\"asset_id\":"+std::to_string(found->second.id)+",\"subimage\":"+num(c.SSE.Xmm1.FP32[0])+",\"x\":"+num(c.SSE.Xmm2.FP32[0])+",\"y\":"+num(c.SSE.Xmm3.FP32[0]);
 if(mode==0){RValue a;if(drawStateGetters[2]){drawStateGetters[2](a,nullptr,nullptr,0,nullptr);s+=",\"alpha\":"+value(a);}s+=",\"xscale\":1,\"yscale\":1,\"angle\":0";}
 if(mode==1){s+=",\"xscale\":"+num(stackFloat(c,0x28))+",\"yscale\":"+num(stackFloat(c,0x30))+",\"angle\":"+num(stackFloat(c,0x38))+",\"alpha\":"+num(stackFloat(c,0x48));}
 if(mode==2){s+=",\"width\":"+num(stackFloat(c,0x28))+",\"height\":"+num(stackFloat(c,0x30))+",\"alpha\":"+num(stackFloat(c,0x40));}
 emit(s+"}");own=false;
}
static std::string readCString(uintptr_t ptr){
 if(!ptr)return "";std::string result;size_t end=0;
 for(size_t i=0;i<1024;i++){if(ptr+i>=end){MEMORY_BASIC_INFORMATION mbi{};if(!VirtualQuery((void*)(ptr+i),&mbi,sizeof mbi)||mbi.State!=MEM_COMMIT||(mbi.Protect&(PAGE_NOACCESS|PAGE_GUARD)))return "<unreadable>";end=(uintptr_t)mbi.BaseAddress+mbi.RegionSize;}
 char ch=*((const char*)(ptr+i));if(!ch)break;if((unsigned char)ch<32&&ch!='\n'&&ch!='\r'&&ch!='\t')return "<non-text>";result+=ch;}return result;
}
static void engineText(ProcessorContext64& c,bool transformed){if(!active||own)return;own=true;
 std::string s="{\"event\":\"native_text\","+context()+",\"x\":"+num(c.SSE.Xmm0.FP32[0])+",\"y\":"+num(c.SSE.Xmm1.FP32[0])+",\"text\":"+quote(readCString(c.R8))+drawState();
 if(transformed)s+=",\"xscale\":"+num(stackFloat(c,0x30))+",\"yscale\":"+num(stackFloat(c,0x38))+",\"angle\":"+num(stackFloat(c,0x40));
 emit(s+"}");own=false;
}
static void textPlain(ProcessorContext64& c){engineText(c,false);}static void textTransformed(ProcessorContext64& c){engineText(c,true);}
static void plain(ProcessorContext64& c){engine(c,0);}static void ext(ProcessorContext64& c){engine(c,1);}static void stretched(ProcessorContext64& c){engine(c,2);}
static void init(){
 fs::create_directories(dir);logFile.open(dir/("HoverInspector-"+std::to_string(GetTickCount64())+".jsonl"));
 uintptr_t base=(uintptr_t)GetModuleHandleW(nullptr);
 int stateIndex=0;for(const char* n:{"draw_get_font","draw_get_color","draw_get_alpha","draw_get_halign","draw_get_valign"}){void* p=nullptr;yy->GetNamedRoutinePointer(n,&p);drawStateGetters[stateIndex++]=(Builtin)p;}
 for(const char* n:{"draw_text","draw_text_ext","draw_text_color","draw_text_colour","draw_text_ext_color","draw_text_ext_colour","draw_text_transformed","draw_text_transformed_color","draw_text_ext_transformed","draw_text_ext_transformed_color","draw_rectangle","draw_rectangle_color","draw_line","draw_line_width","draw_set_font","draw_set_color","draw_set_alpha"}){int index=-1;if(AurieSuccess(yy->GetNamedRoutineIndex(n,&index))){builtinNames[index]=n;logFile<<"{\"event\":\"builtin_index\",\"name\":"<<quote(n)<<",\"index\":"<<index<<"}\n";}}
 using Getter=void*(*)(int);auto getter=(Getter)(base+0x38034c0);
 std::ifstream names(dir/"HoverInspector-sprites.tsv");int id;std::string name;
 while(names>>id>>name){if(yy->CallBuiltin("sprite_exists",{id}).ToBoolean()){auto ptr=getter(id);if(ptr)sprites[(uintptr_t)ptr]={id,name};}}
 auto status=[&](const char* n,AurieStatus st){logFile<<"{\"event\":\"hook\",\"name\":"<<quote(n)<<",\"status\":"<<int(st)<<"}\n";};
 status("HiScores.Draw",cb->RegisterCodeEventCallback(mod,"gml_Object_obj_HiScores_Draw_0",beforeDraw,afterDraw));
 status("text",cb->RegisterScriptFunctionCallback(mod,"gml_Script_draw_text_scribble",text,nullptr,nullptr));
 status("text_ext",cb->RegisterScriptFunctionCallback(mod,"gml_Script_draw_text_scribble_ext",textExt,nullptr,nullptr));
 status("outline",cb->RegisterScriptFunctionCallback(mod,"gml_Script_draw_text_outline",outline,nullptr,nullptr));
 status("scribble_element",cb->RegisterScriptFunctionCallback(mod,"gml_Script_draw@anon@4568@__scribble_class_element@__scribble_class_element",nullptr,scribbleElement,nullptr));
 status("hover",cb->RegisterScriptFunctionCallback(mod,"gml_Script_MouseOverButton",nullptr,hover,nullptr));
 status("sprite",MmCreateMidfunctionHook(module,"HoverInspector.sprite",(void*)(base+0x37f5f50),plain));
 status("sprite_ext",MmCreateMidfunctionHook(module,"HoverInspector.sprite_ext",(void*)(base+0x37efd10),ext));
 status("sprite_stretched",MmCreateMidfunctionHook(module,"HoverInspector.sprite_stretched",(void*)(base+0x37f6540),stretched));
 status("scribble_entry",MmCreateMidfunctionHook(module,"HoverInspector.scribble_entry",(void*)(base+0x446510),scribbleEntry));
 status("yyc_dispatch",MmCreateMidfunctionHook(module,"HoverInspector.dispatch",(void*)(base+0x3760470),dispatchEntry));
 status("text_engine",MmCreateMidfunctionHook(module,"HoverInspector.text",(void*)(base+0x39070b0),textPlain));
 status("text_transformed_engine",MmCreateMidfunctionHook(module,"HoverInspector.text_transformed",(void*)(base+0x39076b0),textTransformed));
 logFile.flush();remaining=120;
}
static void runnerInit(FunctionWrapper<void(int)>&){if(AurieSuccess(ObGetInterface("callbackManager",reinterpret_cast<AurieInterfaceBase*&>(cb)))&&cb)cb->RegisterInitFunction(init);}
EXPORTED AurieStatus ModulePreinitialize(AurieModule* m,const fs::path& p){module=m;dir=p.parent_path().parent_path().parent_path()/"Logs";yy=GetInterface();if(!yy)return AURIE_MODULE_DEPENDENCY_NOT_RESOLVED;return yy->CreateCallback(m,EVENT_RUNNER_INIT,runnerInit,0);}
EXPORTED AurieStatus ModuleInitialize(AurieModule*,const fs::path&){return AURIE_SUCCESS;}
