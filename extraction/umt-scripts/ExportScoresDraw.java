import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import java.io.*;
public class ExportScoresDraw extends GhidraScript {
 public void run() throws Exception {
 File dir=new File(getScriptArgs()[0]);dir.mkdirs();
 Function f=getFunctionAt(toAddr("1423ed130"));
 try(PrintWriter p=new PrintWriter(new File(dir,"Draw_0.asm.tsv"))){InstructionIterator it=currentProgram.getListing().getInstructions(f.getBody(),true);while(it.hasNext()){Instruction i=it.next();p.println(i.getAddress()+"\t"+i);}}
 DecompInterface d=new DecompInterface();d.openProgram(currentProgram);d.setSimplificationStyle("decompile");
 try{DecompileResults r=d.decompileFunction(f,240,monitor);if(r.decompileCompleted() && r.getDecompiledFunction()!=null){try(PrintWriter p=new PrintWriter(new File(dir,"Draw_0.c"))){p.println(r.getDecompiledFunction().getC());}println("SCORES_DRAW_EXPORTED");}else println("SCORES_DRAW_FAILED "+r.getErrorMessage());}finally{d.dispose();}
 }
}
