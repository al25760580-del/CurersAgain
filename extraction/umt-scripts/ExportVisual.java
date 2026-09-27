import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;
public class ExportVisual extends GhidraScript {
 public void run() throws Exception {
 String b=getScriptArgs()[0]; File dir=new File(b,"exports-visual");dir.mkdirs();
 DecompInterface d=new DecompInterface();d.openProgram(currentProgram);
 HashSet<String> done=new HashSet<>();
 try(PrintWriter refs=new PrintWriter(new File(dir,"calls.tsv"))) {
 for(String line:Files.readAllLines(Path.of(b,"visual-functions.tsv"))) {
 String[] v=line.split("\t"); Function f=getFunctionAt(toAddr(v[0]));
 if(f==null || (v.length>2 && !f.getBody().contains(toAddr(v[2])))) {println("REJECT "+line);continue;}
 LinkedHashMap<Function,String> targets=new LinkedHashMap<>(); targets.put(f,v[1]);
 for(Function g:f.getCalledFunctions(monitor)) {
 refs.println(v[1]+"\t"+g.getEntryPoint()+"\t"+g.getName());
 if(v[1].startsWith("builtin_") && !g.isExternal())targets.put(g,g.getName());
 }
 for(Map.Entry<Function,String> t:targets.entrySet()) {
 String id=t.getKey().getEntryPoint().toString();if(!done.add(id))continue;
 DecompileResults r=d.decompileFunction(t.getKey(),25,monitor);
 if(r.decompileCompleted()) {try(PrintWriter p=new PrintWriter(new File(dir,t.getValue()+".c"))){p.println("/* Entry "+id+" */");p.println(r.getDecompiledFunction().getC());}println("VISUAL_EXPORTED "+t.getValue());}
 else println("VISUAL_FAILED "+t.getValue());
 }
 }
 }finally{d.dispose();}
 println("VISUAL_EXPORT_COMPLETE");
 }
}
